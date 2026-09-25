//! Commander: the cards the **Creative Energy** precon (M3C, Satya,
//! Aetherflux Genius) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_satya.rs`.
//!
//! Residuals (each also on its card):
//! - **Cayth, Famed Mechanist** — other nontoken creatures don't gain
//!   fabricate.
//! - **Filigree Racer** — the granted "jump-start" is a flashback for the
//!   card's mana cost (no discard).
//! - **Hourglass of the Lost** — it removes all its time counters (X is that
//!   number), not a chosen X.
//! - **Overclocked Electromancer** — the excess-damage energy isn't gained.
//! - **Razorfield Ripper** — reconfigure costs only {2} (not the {E}{E}{E}
//!   option).
//! - **Sphinx of the Revelation** — the {E} is paid as the ability resolves
//!   (any amount), not as a cost.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, fabricate, on_attack, on_you_attack, target_filtered, unearth};
use crate::effect::{DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, r, u, w, Color, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn vehicle(name: &'static str, mana: ManaCost, p: i32, t: i32, crew: u32) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Crew(crew)],
        ..artifact(name, mana)
    }
}

fn energy(n: i32) -> Effect {
    Effect::AddEnergy(Value::Const(n))
}

fn yours() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn on_your_combat(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
        effect,
    }
}

fn token(name: &str, p: i32, t: i32, types: Vec<CreatureType>, card_types: Vec<CardType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

/// Satya, Aetherflux Genius — menace, haste; attacking copies another
/// nontoken creature of yours tapped and attacking and gets {E}{E}; the copy
/// is sacrificed at the next end step unless you pay {E} equal to its mana
/// value.
pub fn satya_aetherflux_genius() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: yours().and(R::OtherThanSource).and(R::NotToken),
                effect: Box::new(Effect::Seq(vec![
                    Effect::TokenCopyTappedAttacking { source: Selector::Target(0) },
                    Effect::DelayUntilWithCapture {
                        kind: DelayedTriggerKind::NextEndStep,
                        capture: Selector::LastCreatedToken,
                        body: Box::new(Effect::PayEnergyOrElseValue {
                            amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                            otherwise: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
                        }),
                    },
                ])),
            },
            energy(2),
        ]))],
        ..creature(
            "Satya, Aetherflux Genius",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            5,
        )
    }
}

/// Aether Refinery — your energy gains double; tap for {E}, then pay any
/// {E} for an X/X Aetherborn.
pub fn aether_refinery() -> CardDefinition {
    let aetherborn = token("Aetherborn", 0, 0, vec![CreatureType::Aetherborn], vec![CardType::Creature], vec![]);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would get one or more {E}, you get twice that many {E} instead.",
            effect: StaticEffect::EnergyGainDoubles,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                energy(1),
                Effect::PayAnyEnergy {
                    then: Box::new(Effect::If {
                        cond: Predicate::ValueAtLeast(Value::EnergyPaidThisEffect, Value::ONE),
                        then: Box::new(Effect::Seq(vec![
                            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: aetherborn },
                            Effect::SetBasePT {
                                what: Selector::LastCreatedToken,
                                power: Value::EnergyPaidThisEffect,
                                toughness: Value::EnergyPaidThisEffect,
                                duration: Duration::Permanent,
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Aether Refinery", cost(&[generic(4), r(), r()]))
    }
}

/// Aethergeode Miner — attacking gets {E}{E}; pay {E}{E}: blink it.
pub fn aethergeode_miner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(energy(2))],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 2,
            effect: Effect::ExileAndReturnToOwner { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Aethergeode Miner", cost(&[generic(1), w()]), vec![CreatureType::Dwarf, CreatureType::Scout], 3, 1)
    }
}

/// Aethersphere Harvester — flying Vehicle; {E}{E} on entering; pay {E}:
/// lifelink; crew 1.
pub fn aethersphere_harvester() -> CardDefinition {
    let mut d = vehicle("Aethersphere Harvester", cost(&[generic(3)]), 3, 5, 1);
    d.keywords.push(Keyword::Flying);
    d.triggered_abilities = vec![etb(energy(2))];
    d.activated_abilities = vec![ActivatedAbility {
        energy_cost: 1,
        effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
        ..Default::default()
    }];
    d
}

/// Aetherstorm Roc — flying; creatures of yours entering give {E}; attacking,
/// pay {E}{E} for a counter and to tap a defender's creature.
pub fn aetherstorm_roc() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                ),
                effect: energy(1),
            },
            on_attack(Effect::PayEnergy {
                amount: 2,
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Creature.and(R::ControlledByDefendingPlayer),
                        effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
                    },
                ])),
            }),
        ],
        ..creature("Aetherstorm Roc", cost(&[generic(2), w(), w()]), vec![CreatureType::Bird], 3, 3)
    }
}

/// Aurora Shifter — combat damage to a player gets that much {E}; each of
/// your combats, pay {E}{E} to become a copy of another creature of yours,
/// keeping these abilities.
pub fn aurora_shifter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddEnergy(Value::TriggerEventAmount),
            },
            on_your_combat(Effect::PayEnergy {
                amount: 2,
                then: Box::new(Effect::BecomeCopyOf {
                    what: Selector::This,
                    source: target_filtered(yours().and(R::OtherThanSource)),
                    extra_creature_types: vec![],
                    keep_own_triggered: true,
                    keep_own_activated: false,
                }),
            }),
        ],
        ..creature("Aurora Shifter", cost(&[generic(1), u()]), vec![CreatureType::Shapeshifter], 1, 3)
    }
}

/// Blaster Hulk — {1} less per {E} you paid or lost this turn; haste;
/// attacking gets {E}{E}, then eight {E} deal 8 damage divided.
pub fn blaster_hulk() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Haste],
        self_cost_reduction_per: Some((Value::EnergySpentThisTurn(PlayerRef::You), 1)),
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            energy(2),
            Effect::PayEnergy {
                amount: 8,
                then: Box::new(Effect::DealDamageDivided {
                    total: Value::Const(8),
                    filter: R::Creature.or(R::Player).or(R::Planeswalker),
                    max_targets: 8,
                    retaliate_to_source: false,
                }),
            },
        ]))],
        ..creature("Blaster Hulk", cost(&[generic(6), r(), r()]), vec![CreatureType::Pirate], 8, 8)
    }
}

/// Cayth, Famed Mechanist — fabricate 1; {2}, {T}: populate or proliferate.
///
/// Residual: other nontoken creatures don't gain fabricate.
pub fn cayth_famed_mechanist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![fabricate(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::ChooseMode(vec![Effect::Populate { who: PlayerRef::You }, Effect::Proliferate]),
            ..Default::default()
        }],
        ..creature(
            "Cayth, Famed Mechanist",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Dwarf, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Conversion Apparatus — {C}; {3}, {T}: {E}{E}{E}; {T}, pay {E}{E}{E}: three
/// mana in any colors.
pub fn conversion_apparatus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility { mana_cost: cost(&[generic(3)]), tap_cost: true, effect: energy(3), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 3,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(3)) },
                ..Default::default()
            },
        ],
        ..artifact("Conversion Apparatus", cost(&[generic(3)]))
    }
}

/// Filigree Racer — Vehicle; {E}{E}{E}{E} on entering; attacking, pay {E}{E}
/// to let an instant or sorcery in your graveyard be cast again this turn;
/// crew 1.
///
/// Residual: the grant is a flashback for the card's mana cost (no
/// discard).
pub fn filigree_racer() -> CardDefinition {
    let mut d = vehicle("Filigree Racer", cost(&[generic(3), r()]), 5, 5, 1);
    d.triggered_abilities = vec![
        etb(energy(4)),
        on_attack(Effect::PayEnergy {
            amount: 2,
            then: Box::new(Effect::GrantFlashbackThisTurn {
                what: target_filtered(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard),
                ),
            }),
        }),
    ];
    d
}

/// Gonti's Aether Heart — your artifacts entering give {E}{E}; pay eight
/// {E} and exile it for an extra turn.
pub fn gontis_aether_heart() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: energy(2),
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 8,
            exile_self_cost: true,
            effect: Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Gonti's Aether Heart", cost(&[generic(6)]))
    }
}

/// Hourglass of the Lost — {T}: {W} and a time counter; {T}, exile it:
/// return each nonland permanent card with mana value X (its time
/// counters) from your graveyard.
///
/// Residual: X is all its time counters, not a chosen number.
pub fn hourglass_of_the_lost() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::White, Value::ONE) },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Time, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::WithX {
                    x: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Time },
                    body: Box::new(Effect::Seq(vec![
                        Effect::Exile { what: Selector::This },
                        Effect::Move {
                            what: Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: Zone::Graveyard,
                                filter: R::PermanentCard.and(R::Not(Box::new(R::Land))).and(R::ManaValueExactlyXFromCost),
                            },
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                    ])),
                },
                ..Default::default()
            },
        ],
        ..artifact("Hourglass of the Lost", cost(&[generic(2), w()]))
    }
}

/// Localized Destruction — {E}, pay any {E} to keep creatures of yours with
/// that much power, then destroy all creatures.
pub fn localized_destruction() -> CardDefinition {
    CardDefinition {
        name: "Localized Destruction",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            energy(1),
            Effect::PayAnyEnergy {
                then: Box::new(Effect::WithX {
                    x: Value::EnergyPaidThisEffect,
                    body: Box::new(Effect::GrantKeyword {
                        what: Selector::EachPermanent(yours().and(R::PowerExactlyXFromCost)),
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    }),
                }),
            },
            Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
        ]),
        ..Default::default()
    }
}

/// Overclocked Electromancer — each of your combats, pay {E}{E}{E} for a
/// counter; attacking doubles its power.
///
/// Residual: the excess-damage energy isn't gained.
pub fn overclocked_electromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_your_combat(Effect::PayEnergy {
                amount: 3,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            }),
            on_attack(Effect::DoublePower { what: Selector::This, times: Value::ONE, duration: Duration::EndOfTurn }),
        ],
        ..creature(
            "Overclocked Electromancer",
            cost(&[generic(2), r()]),
            vec![CreatureType::Lizard, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Razorfield Ripper — it or the creature it equips attacking gets {E},
/// then +X/+X for your {E}; reconfigure {2}.
///
/// Residual: reconfigure costs only {2}.
pub fn razorfield_ripper() -> CardDefinition {
    let surge = || {
        on_attack(Effect::Seq(vec![
            energy(1),
            Effect::PumpPT {
                what: Selector::This,
                power: Value::EnergyOf(PlayerRef::You),
                toughness: Value::EnergyOf(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
        ]))
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Reconfigure(cost(&[generic(2)]))],
        triggered_abilities: vec![surge()],
        equipped_bonus: Some(EquipBonus { triggered_abilities: vec![surge()], ..Default::default() }),
        ..creature("Razorfield Ripper", cost(&[generic(2), w()]), vec![CreatureType::Rhino], 3, 3)
    }
}

/// Salvation Colossus — flying, vigilance, trample; attacking pumps and
/// protects your other creatures; unearth by paying eight {E}.
pub fn salvation_colossus() -> CardDefinition {
    let others = || Selector::EachPermanent(yours().and(R::OtherThanSource));
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![on_you_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: others(),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: others(), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
        ]))],
        activated_abilities: vec![ActivatedAbility { energy_cost: 8, ..unearth(ManaCost::default()) }],
        ..creature("Salvation Colossus", cost(&[generic(6), w(), w()]), vec![CreatureType::Construct], 9, 9)
    }
}

/// Sphinx of the Revelation — flying, lifelink; life you gain becomes {E};
/// {W}{U}{U}, {T}, pay X {E}: draw X.
///
/// Residual: the {E} is paid as the ability resolves.
pub fn sphinx_of_the_revelation() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddEnergy(Value::TriggerEventAmount),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), u()]),
            tap_cost: true,
            effect: Effect::PayAnyEnergy {
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::EnergyPaidThisEffect }),
            },
            ..Default::default()
        }],
        ..creature("Sphinx of the Revelation", cost(&[generic(3), w(), u()]), vec![CreatureType::Sphinx], 4, 5)
    }
}

/// Stone Idol Generator — each attacker of yours gets {E}; {T}, pay six
/// {E}: a 6/12 trampling Construct.
pub fn stone_idol_generator() -> CardDefinition {
    let construct = token(
        "Construct",
        6,
        12,
        vec![CreatureType::Construct],
        vec![CardType::Artifact, CardType::Creature],
        vec![Keyword::Trample],
    );
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: energy(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 6,
            sorcery_speed: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: construct },
            ..Default::default()
        }],
        ..artifact("Stone Idol Generator", cost(&[generic(5)]))
    }
}
