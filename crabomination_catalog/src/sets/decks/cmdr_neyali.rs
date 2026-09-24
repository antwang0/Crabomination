//! Commander: the cards the **Rebellion Rising** precon (ONC, Neyali, Suns'
//! Vanguard) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Collective Effort** — escalate is paid as it resolves (the engine's
//!   standing `Escalate` approximation), tapping your first untapped creature.
//! - **Goldwardens' Gambit** — each token takes your highest-mana-value
//!   *unattached* Equipment (no pick); an attached one is never moved.
//! - **Neyali** — "tokens attack a player" counts tokens attacking a
//!   planeswalker an opponent controls too.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EquipScale, Keyword, LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate,
    ZoneDest,
};
use crate::mana::{Color, ManaCost, cost, generic, r, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn token(
    name: &str,
    color: Color,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    keywords: Vec<Keyword>,
) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    })
}

fn rebel() -> Arc<TokenDefinition> {
    token("Rebel", Color::Red, vec![CreatureType::Rebel], 2, 2, vec![])
}

fn soldier() -> Arc<TokenDefinition> {
    token("Soldier", Color::White, vec![CreatureType::Soldier], 1, 1, vec![])
}

fn make(who: PlayerRef, count: Value, definition: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who, count, definition }
}

fn your_tokens() -> R {
    R::Creature.and(R::IsToken).and(R::ControlledByYou)
}

fn equipment() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Equipment)
}

/// Neyali, Suns' Vanguard — attacking tokens have double strike; a token
/// attack exiles your top card, playable on any turn you attack with a token.
pub fn neyali_suns_vanguard() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Attacking tokens you control have double strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::IsToken.and(R::ControlledByYou).and(R::IsAttacking),
                ),
                keyword: Keyword::DoubleStrike,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsToken.and(R::IsAttackingAnOpponent),
                })
                .once_per_batch(),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::TurnsHolderAttacksWithAToken { holder: 0 },
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..legendary(creature(
            "Neyali, Suns' Vanguard",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            3,
            3,
        ))
    }
}

/// Call the Coppercoats — strive; a 1/1 Human Soldier per creature the
/// targeted opponents control.
pub fn call_the_coppercoats() -> CardDefinition {
    CardDefinition {
        name: "Call the Coppercoats",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        cost_per_extra_target: Some(cost(&[generic(1), w()])),
        effect: Effect::ApplyToTargets {
            max_targets: 10,
            min_targets: 0,
            filter: R::OpponentPlayer,
            effect: Box::new(make(
                PlayerRef::You,
                Value::CreatureCountControlledBy(PlayerRef::Target(0)),
                token(
                    "Human Soldier",
                    Color::White,
                    vec![CreatureType::Human, CreatureType::Soldier],
                    1,
                    1,
                    vec![],
                ),
            )),
        },
        ..Default::default()
    }
}

/// Collective Effort — escalate (tap an untapped creature you control):
/// destroy a power 4+ creature; destroy an enchantment; +1/+1 counters on a
/// target player's creatures.
pub fn collective_effort() -> CardDefinition {
    CardDefinition {
        name: "Collective Effort",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Escalate {
            cost: Box::new(Effect::TapUpToValue {
                count: Value::ONE,
                filter: R::Creature.and(R::ControlledByYou).and(R::Untapped),
                skip_untap: false,
                exact: true,
            }),
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
                Effect::Destroy { what: target_filtered(R::Enchantment) },
                Effect::AddCounter {
                    what: Selector::ControlledBy {
                        who: PlayerRef::Target(0),
                        filter: R::Creature,
                    },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ],
        },
        ..Default::default()
    }
}

/// Elspeth Tirel — +2: 1 life per creature you control; −2: three Soldiers;
/// −5: destroy all other permanents but lands and tokens.
pub fn elspeth_tirel() -> CardDefinition {
    CardDefinition {
        name: "Elspeth Tirel",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::CreatureCountControlledBy(PlayerRef::You),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: make(PlayerRef::You, Value::Const(3), soldier()),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::OtherThanSource.and(R::Not(Box::new(R::Land))).and(R::NotToken),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Glimmer Lens — For Mirrodin!; when the equipped creature and at least one
/// other creature attack, draw a card.
pub fn glimmer_lens() -> CardDefinition {
    crate::sets::one::for_mirrodin(
        "Glimmer Lens",
        cost(&[generic(1), w()]),
        cost(&[generic(1), w()]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                    Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 2 },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            }],
            ..Default::default()
        },
    )
}

/// Hexplate Wallbreaker — For Mirrodin!; +2/+2; its first-combat attack
/// untaps every attacker and adds a combat phase.
pub fn hexplate_wallbreaker() -> CardDefinition {
    crate::sets::one::for_mirrodin(
        "Hexplate Wallbreaker",
        cost(&[generic(3), r(), r()]),
        cost(&[generic(3), r()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::IsFirstCombatPhaseThisTurn,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap {
                            what: Selector::EachPermanent(R::IsAttacking.and(R::ControlledByYou)),
                            up_to: None,
                        },
                        Effect::AdditionalCombatPhase { count: Value::ONE },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            }],
            ..Default::default()
        },
    )
}

/// Kemba's Banner — For Mirrodin!; +1/+1 per creature you control.
pub fn kembas_banner() -> CardDefinition {
    crate::sets::one::for_mirrodin(
        "Kemba's Banner",
        cost(&[generic(3), w()]),
        cost(&[generic(2), w()]),
        EquipBonus {
            scale: Some(EquipScale {
                filter: R::Creature.and(R::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Goldwardens' Gambit — affinity for Equipment; five hasty 2/2 Rebels, each
/// taking your best unattached Equipment.
pub fn goldwardens_gambit() -> CardDefinition {
    CardDefinition {
        name: "Goldwardens' Gambit",
        cost: cost(&[generic(6), r(), r()]),
        card_types: vec![CardType::Sorcery],
        affinity_filter: Some(equipment()),
        effect: Effect::Seq(vec![
            make(PlayerRef::You, Value::Const(5), rebel()),
            Effect::GrantKeyword {
                what: Selector::LastCreatedTokens,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::ForEach {
                selector: Selector::LastCreatedTokens,
                body: Box::new(Effect::If {
                    cond: Predicate::SelectorExists(Selector::EachPermanent(
                        equipment().and(R::ControlledByYou).and(R::Unattached),
                    )),
                    then: Box::new(Effect::Attach {
                        what: Selector::GreatestManaValueControlledMatching {
                            who: PlayerRef::You,
                            filter: equipment().and(R::Unattached),
                        },
                        to: Selector::TriggerSource,
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Harmonious Archon — flying; non-Archon creatures have base P/T 3/3; two
/// 1/1 Humans on entry.
pub fn harmonious_archon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Non-Archon creatures have base power and toughness 3/3.",
            effect: StaticEffect::SetBasePtForFilter {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Archon)))),
                ),
                power: 3,
                toughness: 3,
            },
        }],
        triggered_abilities: vec![etb(make(
            PlayerRef::You,
            Value::Const(2),
            token("Human", Color::White, vec![CreatureType::Human], 1, 1, vec![]),
        ))],
        ..creature(
            "Harmonious Archon",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Archon],
            4,
            5,
        )
    }
}

/// Hate Mirage — hasty token copies of up to two creatures you don't
/// control, exiled at the next end step.
pub fn hate_mirage() -> CardDefinition {
    CardDefinition {
        name: "Hate Mirage",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
            effect: Box::new(Effect::CreateTokenCopiesHasteSac {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::Target(0),
                exile: true,
            }),
        },
        ..Default::default()
    }
}

/// Mace of the Valiant — +1/+1 per charge counter and vigilance; a charge
/// counter whenever a creature you control enters.
pub fn mace_of_the_valiant() -> CardDefinition {
    CardDefinition {
        name: "Mace of the Valiant",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Vigilance],
            scale: Some(EquipScale {
                per_power: 1,
                per_toughness: 1,
                count_self_counters: Some(CounterType::Charge),
                ..Default::default()
            }),
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Otharri, Suns' Glory — flying, lifelink, haste; each attack adds an
/// experience counter and that many attacking Rebels; returns from the
/// graveyard for {2}{R}{W} and a tapped Rebel.
pub fn otharri_suns_glory() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::AddExperience(Value::ONE),
            Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ControllerExperience,
                definition: rebel(),
                cleanup: Default::default(),
                defender: None,
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), w()]),
            tap_other_filter: Some(R::HasCreatureType(CreatureType::Rebel).and(R::ControlledByYou)),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..legendary(creature(
            "Otharri, Suns' Glory",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Phoenix],
            3,
            3,
        ))
    }
}

/// Prava of the Steel Legion — your creature tokens get +1/+4 on your turn;
/// {3}{W}: a 1/1 Soldier; partner.
pub fn prava_of_the_steel_legion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        static_abilities: vec![StaticAbility {
            description: "During your turn, creature tokens you control get +1/+4.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(your_tokens()),
                    power: 1,
                    toughness: 4,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: make(PlayerRef::You, Value::ONE, soldier()),
            ..Default::default()
        }],
        ..legendary(creature(
            "Prava of the Steel Legion",
            cost(&[generic(2), w()]),
            vec![CreatureType::Cat, CreatureType::Soldier],
            1,
            4,
        ))
    }
}

/// Roar of Resistance — your creature tokens have haste; when creatures
/// attack, you may pay {1}{R} for +2/+0 on those attacking your opponents.
pub fn roar_of_resistance() -> CardDefinition {
    CardDefinition {
        name: "Roar of Resistance",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(your_tokens()),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay {1}{R}: creatures attacking your opponents get +2/+0?".into(),
                mana_cost: cost(&[generic(1), r()]),
                body: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(R::IsAttackingAnOpponent),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Silverwing Squadron — flying, vigilance, */* = creatures you control;
/// each attack makes a vigilant 2/2 Knight per opponent.
pub fn silverwing_squadron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Its power and toughness are each equal to the number of creatures you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Creature.and(R::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![on_attack(make(
            PlayerRef::You,
            Value::OpponentCount,
            token("Knight", Color::White, vec![CreatureType::Knight], 2, 2, vec![Keyword::Vigilance]),
        ))],
        ..creature(
            "Silverwing Squadron",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            0,
            0,
        )
    }
}

/// Staff of the Storyteller — a 1/1 flying Spirit on entry; a story counter
/// per creature-token batch; {W}, {T}, remove one: draw.
pub fn staff_of_the_storyteller() -> CardDefinition {
    CardDefinition {
        name: "Staff of the Storyteller",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(make(
                PlayerRef::You,
                Value::ONE,
                token("Spirit", Color::White, vec![CreatureType::Spirit], 1, 1, vec![Keyword::Flying]),
            )),
            TriggeredAbility {
                event: EventSpec::new(EventKind::TokenCreated, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    })
                    .once_per_batch(),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Story,
                    amount: Value::ONE,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Story, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vulshok Factory — {T}: {R} and a charge counter; {2}{R}, {T}, sacrifice:
/// an X/X hasty Golem, X its charge counters (sorcery speed).
pub fn vulshok_factory() -> CardDefinition {
    let x = Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge };
    CardDefinition {
        name: "Vulshok Factory",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colors(vec![Color::Red]),
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: make(
                    PlayerRef::You,
                    Value::ONE,
                    Arc::new(TokenDefinition {
                        name: "Golem".into(),
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Golem],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Haste],
                        dynamic_pt: Some((x.clone(), x)),
                        ..Default::default()
                    }),
                ),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
