//! Commander: the cards the **Living Energy** precon (DRC, Saheeli, Radiant
//! Creator) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_saheeli_radiant.rs`.
//!
//! Residuals (each also on its card):
//! - **Aetherflux Conduit** — the free casts last the turn, not only the
//!   resolution.
//! - **Territorial Aetherkite** / **Rampaging Aetherhood** — "one or more
//!   {E}" is paid in full by the headless seat.
//! - **Saheeli, Radiant Creator** — the copy's target is picked as the
//!   trigger goes on the stack, not in a reflexive trigger.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, cost, g, generic, r, u, x};
use std::sync::Arc;

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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn energy(n: i32) -> Effect {
    Effect::AddEnergy(Value::Const(n))
}

fn energy_ability(n: u32, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { energy_cost: n, effect, ..Default::default() }
}

/// "Whenever a land you control enters" (landfall — any entry, not only a
/// land play).
fn landfall(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
        effect,
    }
}

fn thopter() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    }
}

fn make(definition: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(definition) }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// Saheeli, Radiant Creator — casting an Artificer or artifact spell gets
/// {E}; at the beginning of combat on your turn you may pay {E}{E}{E} for a
/// hasty 5/5 artifact-creature token copy of a permanent you control,
/// sacrificed at the next end step.
pub fn saheeli_radiant_creator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact.or(R::HasCreatureType(CreatureType::Artificer)),
                    },
                ),
                effect: energy(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Pay {E}{E}{E} to copy a permanent you control?".into(),
                    body: Box::new(Effect::PayEnergy {
                        amount: 3,
                        then: Box::new(Effect::Seq(vec![
                            Effect::CreateTokenCopyOf {
                                who: PlayerRef::You,
                                count: Value::ONE,
                                source: target_filtered(R::Permanent.and(R::ControlledByYou)),
                                extra_creature_types: vec![],
                                extra_card_types: vec![CardType::Artifact, CardType::Creature],
                                override_pt: Some((5, 5)),
                                override_colors: None,
                                enters_tapped: false,
                                non_legendary: false,
                                legendary: false,
                                extra_keywords: vec![Keyword::Haste],
                            },
                            Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedToken },
                        ])),
                    }),
                },
            },
        ],
        ..legendary(creature(
            "Saheeli, Radiant Creator",
            cost(&[generic(1), g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            4,
            4,
        ))
    }
}

/// Aetherflux Conduit — every spell you cast gets {E} per mana spent; {T},
/// pay fifty {E}: draw seven, then cast spells from your hand for free.
/// Residual: the free casts last the turn.
pub fn aetherflux_conduit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::AddEnergy(Value::CastSpellManaSpent),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 50,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(7) },
                Effect::FreeSpellsFromHandThisTurn,
            ]),
            ..Default::default()
        }],
        ..artifact("Aetherflux Conduit", cost(&[generic(6)]))
    }
}

/// Aetheric Amplifier — {T}: any color; {4}, {T} (sorcery): double each kind
/// of counter on target permanent, or each kind you have.
pub fn aetheric_amplifier() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::ChooseMode(vec![
                    Effect::DoubleAllCountersOn { what: target_filtered(R::Permanent) },
                    Effect::DoublePlayerCounters { who: PlayerRef::You },
                ]),
                ..Default::default()
            },
        ],
        ..artifact("Aetheric Amplifier", cost(&[generic(3)]))
    }
}

/// Aethersquall Ancient — flying; your upkeep: {E}{E}{E}; pay eight {E}
/// (sorcery): every other creature back to its owner's hand.
pub fn aethersquall_ancient() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: energy(3),
        }],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            ..energy_ability(
                8,
                Effect::Move {
                    what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            )
        }],
        ..creature("Aethersquall Ancient", cost(&[generic(5), u(), u()]), vec![CreatureType::Leviathan], 6, 6)
    }
}

/// Aethertide Whale — flying; enters with six {E}; pay four {E}: back to hand.
pub fn aethertide_whale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(energy(6))],
        activated_abilities: vec![energy_ability(
            4,
            Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
        )],
        ..creature("Aethertide Whale", cost(&[generic(4), u(), u()]), vec![CreatureType::Whale], 6, 4)
    }
}

/// Aetherwind Basker — trample; entering or attacking gets {E} per creature
/// you control; pay {E}: +1/+1 until end of turn.
pub fn aetherwind_basker() -> CardDefinition {
    let per_creature = || Effect::AddEnergy(Value::CountOf(Box::new(yours(R::Creature))));
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(per_creature()), on_attack(per_creature())],
        activated_abilities: vec![energy_ability(
            1,
            Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature("Aetherwind Basker", cost(&[generic(4), g(), g(), g()]), vec![CreatureType::Lizard], 7, 7)
    }
}

/// Aetherworks Marvel — a permanent of yours going to a graveyard gets {E};
/// {T}, pay six {E}: look at the top six, cast one free, the rest to the
/// bottom.
pub fn aetherworks_marvel() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl),
            effect: energy(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 6,
            effect: Effect::RevealTopMayCastOneFree { count: Value::Const(6), max_mv: Value::Const(99), filter: None },
            ..Default::default()
        }],
        ..artifact("Aetherworks Marvel", cost(&[generic(4)]))
    }
}

/// Architect of the Untamed — landfall: {E}; pay eight {E}: a 6/6 Beast
/// artifact creature token.
pub fn architect_of_the_untamed() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(energy(1))],
        activated_abilities: vec![energy_ability(
            8,
            make(TokenDefinition {
                name: "Beast".into(),
                power: 6,
                toughness: 6,
                card_types: vec![CardType::Artifact, CardType::Creature],
                subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
                ..Default::default()
            }),
        )],
        ..creature(
            "Architect of the Untamed",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Artificer, CreatureType::Druid],
            2,
            3,
        )
    }
}

/// Confiscation Coup — target artifact or creature; get four {E}, then you
/// may pay {E} equal to its mana value to gain control of it.
pub fn confiscation_coup() -> CardDefinition {
    let it = || target_filtered(R::Artifact.or(R::Creature));
    CardDefinition {
        name: "Confiscation Coup",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            energy(4),
            Effect::PayEnergyValue {
                amount: Value::ManaValueOf(Box::new(it())),
                then: Box::new(Effect::GainControl { what: it(), to: None, duration: Duration::Permanent }),
            },
        ]),
        ..Default::default()
    }
}

/// Decoction Module — a creature of yours entering gets {E}; {4}, {T}:
/// return a creature you control to its owner's hand.
pub fn decoction_module() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: energy(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..artifact("Decoction Module", cost(&[generic(2)]))
    }
}

/// Era of Innovation — an artifact or Artificer of yours entering: you may
/// pay {1} for {E}{E}; pay six {E}, sacrifice it: draw three.
pub fn era_of_innovation() -> CardDefinition {
    CardDefinition {
        name: "Era of Innovation",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.or(R::HasCreatureType(CreatureType::Artificer)),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {1} for {E}{E}?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(energy(2)),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 6,
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lightning Runner — double strike, haste; attacking: {E}{E}, then you may
/// pay eight {E} to untap your creatures and add a combat phase.
pub fn lightning_runner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            energy(2),
            Effect::MayDo {
                description: "Pay eight {E} for an additional combat phase?".into(),
                body: Box::new(Effect::PayEnergy {
                    amount: 8,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: yours(R::Creature), up_to: None },
                        Effect::AdditionalCombatPhase { count: Value::ONE },
                    ])),
                }),
            },
        ]))],
        ..creature(
            "Lightning Runner",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Nissa, Worldsoul Speaker — landfall: {E}{E}; you may pay eight {E} rather
/// than the mana cost of permanent spells you cast.
pub fn nissa_worldsoul_speaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(energy(2))],
        static_abilities: vec![StaticAbility {
            description: "You may pay eight {E} rather than pay the mana cost for permanent spells you cast.",
            effect: StaticEffect::EnergyAlternativeCostForFilter { filter: R::Permanent, energy: 8 },
        }],
        ..legendary(creature(
            "Nissa, Worldsoul Speaker",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            3,
        ))
    }
}

/// Peema Aether-Seer — enters: {E} equal to the greatest power among your
/// creatures; pay {E}{E}{E}: target creature blocks this turn if able.
pub fn peema_aether_seer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::GreatestPowerControlled { who: PlayerRef::You }))],
        activated_abilities: vec![energy_ability(
            3,
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::MustBlock,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature(
            "Peema Aether-Seer",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            2,
        )
    }
}

/// Peema Trailblazer — trample; combat damage to a player gets that much
/// {E}; exhaust — pay six {E}: two +1/+1 counters, then draw equal to the
/// greatest power among your creatures.
pub fn peema_trailblazer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddEnergy(Value::TriggerEventAmount),
        }],
        activated_abilities: vec![ActivatedAbility {
            exhaust: true,
            ..energy_ability(
                6,
                Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::GreatestPowerControlled { who: PlayerRef::You },
                    },
                ]),
            )
        }],
        ..creature(
            "Peema Trailblazer",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elephant, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Pia Nalaar, Chief Mechanic — one or more of your artifact creatures
/// dealing combat damage to a player gets {E}{E}; at your end step you may
/// pay one or more {E} for an X/X flying Vehicle token (crew 2), X the {E}
/// paid.
pub fn pia_nalaar_chief_mechanic() -> CardDefinition {
    let aetherjet = TokenDefinition {
        name: "Nalaar Aetherjet".into(),
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        dynamic_pt: Some((Value::EnergyPaidThisEffect, Value::EnergyPaidThisEffect)),
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec {
                    once_per_batch: true,
                    ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::Creature) },
                    )
                },
                effect: energy(2),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::PayAnyEnergy {
                    then: Box::new(Effect::If {
                        cond: Predicate::ValueAtLeast(Value::EnergyPaidThisEffect, Value::ONE),
                        then: Box::new(make(aetherjet)),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            },
        ],
        ..legendary(creature(
            "Pia Nalaar, Chief Mechanic",
            cost(&[g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            4,
        ))
    }
}

/// Rampaging Aetherhood — trample, ward {2}; your upkeep: {E} equal to its
/// power, then pay any amount of {E} for that many +1/+1 counters.
pub fn rampaging_aetherhood() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddEnergy(Value::PowerOf(Box::new(Selector::This))),
                Effect::PayAnyEnergy {
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::EnergyPaidThisEffect,
                    }),
                },
            ]),
        }],
        ..creature(
            "Rampaging Aetherhood",
            cost(&[generic(4), g()]),
            vec![CreatureType::Snake, CreatureType::Hydra],
            4,
            4,
        )
    }
}

/// Stridehangar Automaton — Thopters you control get +1/+1; your artifact
/// token batches come with an extra 1/1 flying Thopter.
pub fn stridehangar_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![
            StaticAbility {
                description: "Thopters you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: yours(R::Creature.and(R::HasCreatureType(CreatureType::Thopter))),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Artifact tokens you create come with an additional 1/1 Thopter.",
                effect: StaticEffect::ArtifactTokenCreationAddsToken { definition: thopter() },
            },
        ],
        ..creature("Stridehangar Automaton", cost(&[generic(3)]), vec![CreatureType::Construct], 1, 4)
    }
}

/// Territorial Aetherkite — flying, haste; enters: {E}{E}, then pay any
/// amount of {E} to deal that much damage to each other creature.
pub fn territorial_aetherkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            energy(2),
            Effect::PayAnyEnergy {
                then: Box::new(Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                    amount: Value::EnergyPaidThisEffect,
                }),
            },
        ]))],
        ..creature(
            "Territorial Aetherkite",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Cat, CreatureType::Dragon],
            6,
            5,
        )
    }
}

/// Treasure Vault — artifact land; {T}: {C}; {X}{X}, {T}, sacrifice it: X
/// Treasures.
pub fn treasure_vault() -> CardDefinition {
    CardDefinition {
        name: "Treasure Vault",
        card_types: vec![CardType::Artifact, CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[x(), x()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::XFromCost,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Whirler Virtuoso — enters with {E}{E}{E}; pay {E}{E}{E}: a 1/1 flying
/// Thopter.
pub fn whirler_virtuoso() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(energy(3))],
        activated_abilities: vec![energy_ability(3, make(thopter()))],
        ..creature(
            "Whirler Virtuoso",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Vedalken, CreatureType::Artificer],
            2,
            3,
        )
    }
}
