//! Legends (LEG) wave 7 — the set's last creatures, artifacts, Auras and
//! spells. Tests in `classic_sets/leg6`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest, ZoneRef,
    shortcut::{target, target_any, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, c, types, p, t) }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..enchantment(name, c)
    }
}

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn upkeep(scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), scope)
}

fn dies(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect,
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Blazing Effigy — dies for 3, plus whatever earlier copies of itself burned
/// into it this turn.
pub fn blazing_effigy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![dies(Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Sum(vec![
                Value::Const(3),
                Value::DamageToSourceThisTurnFromOthersNamedSame,
            ]),
        })],
        ..creature("Blazing Effigy", cost(&[generic(1), r()]), vec![CreatureType::Elemental], 0, 3)
    }
}

/// Brine Hag — everything that helped kill it shrinks to 0/2, for good.
pub fn brine_hag() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![dies(Effect::SetBasePT {
            what: Selector::EachPermanent(R::Creature.and(R::DealtDamageToSourceThisTurn)),
            power: Value::Const(0),
            toughness: Value::Const(2),
            duration: Duration::Permanent,
        })],
        ..creature("Brine Hag", cost(&[generic(2), u(), u()]), vec![CreatureType::Hag], 2, 2)
    }
}

/// Giant Slug — buys landwalk of a type chosen on your next upkeep.
pub fn giant_slug() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::AtYourNextUpkeep {
                body: Box::new(Effect::Seq(vec![
                    Effect::ChooseBasicLandTypeForSource,
                    Effect::GrantChosenTypeLandwalk { what: Selector::This },
                ])),
            },
            ..Default::default()
        }],
        ..creature("Giant Slug", cost(&[generic(1), b()]), vec![CreatureType::Slug], 1, 1)
    }
}

/// Giant Turtle — attacks only every other turn.
pub fn giant_turtle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackIfAttackedLastTurn],
        ..creature("Giant Turtle", cost(&[generic(1), g(), g()]), vec![CreatureType::Turtle], 2, 4)
    }
}

/// Petra Sphinx — the target player guesses their own top card.
pub fn petra_sphinx() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::NameCardThenRevealTopBin { who: PlayerRef::Target(0) },
            ..Default::default()
        }],
        ..creature(
            "Petra Sphinx",
            cost(&[generic(2), w(), w(), w()]),
            vec![CreatureType::Sphinx],
            3,
            4,
        )
    }
}

/// Sentinel — sets its own toughness off whatever it's fighting.
pub fn sentinel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::SetBasePT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::Sum(vec![
                    Value::Const(1),
                    Value::PowerOf(Box::new(Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::BlockingOrBlockedBySource),
                    })),
                ]),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..creature("Sentinel", cost(&[generic(4)]), vec![CreatureType::Shapeshifter], 1, 1)
    }
}

/// Wall of Dust — whatever it blocks sits out its controller's next turn.
pub fn wall_of_dust() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::CantAttackNextTurn { what: Selector::BlockedAttacker },
        }],
        ..creature("Wall of Dust", cost(&[generic(2), r()]), vec![CreatureType::Wall], 1, 4)
    }
}

/// Halfdane — wears a rival's body from upkeep to upkeep.
pub fn halfdane() -> CardDefinition {
    let other = || Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::OtherThanSource) };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::SetBasePT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(other())),
                toughness: Value::ToughnessOf(Box::new(other())),
                duration: Duration::Permanent,
            },
        }],
        ..legend(
            "Halfdane",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Shapeshifter],
            3,
            3,
        )
    }
}

/// Hazezon Tamar — a delayed desert, exiled when he goes.
pub fn hazezon_tamar() -> CardDefinition {
    let sand_warrior = || TokenDefinition {
        name: "Sand Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::Green, Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sand, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AtYourNextUpkeep {
                    body: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        definition: sand_warrior(),
                        count: Value::CountMatching {
                            sel: Box::new(Selector::EachMatching {
                                zone: ZoneRef::Battlefield,
                                filter: R::Land,
                            }),
                            filter: R::ControlledByYou,
                        },
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Sand)),
                    to: ZoneDest::Exile,
                },
            },
        ],
        ..legend(
            "Hazezon Tamar",
            cost(&[generic(4), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            4,
        )
    }
}

// ── Artifacts ──────────────────────────────────────────────────────────────

/// Sword of the Ages — cash in the whole board for one enormous bolt.
pub fn sword_of_the_ages() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..artifact(
            "Sword of the Ages",
            cost(&[generic(6)]),
            vec![ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                sac_any_number_filter: Some(R::Creature),
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_any(), amount: Value::SacrificedTotalPower },
                    Effect::ExileCostSacrificedBatch,
                ]),
                ..Default::default()
            }],
        )
    }
}

/// Gauntlets of Chaos — a two-way trade that strips both permanents bare.
pub fn gauntlets_of_chaos() -> CardDefinition {
    artifact(
        "Gauntlets of Chaos",
        cost(&[generic(5)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::ExchangeControl {
                    a: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::ControlledByYou
                            .and(R::Artifact.or(R::Creature).or(R::Land)),
                    },
                    b: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::ControlledByOpponent
                            .and(R::Artifact.or(R::Creature).or(R::Land)),
                    },
                },
                Effect::Destroy { what: Selector::AttachedTo(Box::new(Selector::Target(0))) },
                Effect::Destroy { what: Selector::AttachedTo(Box::new(Selector::Target(1))) },
            ]),
            ..Default::default()
        }],
    )
}

// ── Enchantments and Auras ─────────────────────────────────────────────────

/// Backfire — the enchanted creature's blows come back at its own controller.
pub fn backfire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentSourceDamagedYou)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsHostOfSource,
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(host()))),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..aura("Backfire", cost(&[u()]), R::Creature)
    }
}

/// Dream Coat — repaints its host once a turn.
pub fn dream_coat() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            effect: Effect::BecomeChosenColor { what: host(), duration: Duration::Permanent },
            ..Default::default()
        }],
        ..aura("Dream Coat", cost(&[u()]), R::Creature)
    }
}

/// Greater Realm of Preservation — a Circle of Protection for two colors.
pub fn greater_realm_of_preservation() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::HasColor(Color::Black).or(R::HasColor(Color::Red)),
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..enchantment("Greater Realm of Preservation", cost(&[generic(1), w()]))
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Enchantment Alteration — walk an Aura off its host onto a new one.
pub fn enchantment_alteration() -> CardDefinition {
    CardDefinition {
        name: "Enchantment Alteration",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ReattachTargetAura {
            aura: Selector::TargetFiltered {
                slot: 0,
                filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
            },
            to: Selector::TargetFiltered { slot: 1, filter: R::Creature.or(R::Land) },
        },
        ..Default::default()
    }
}

/// Eureka — everyone empties their permanents onto the table at once.
pub fn eureka() -> CardDefinition {
    CardDefinition {
        name: "Eureka",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerMayPutPermanentFromHand {
            filter: R::Any,
            others_only: false,
            repeat: true,
        },
        ..Default::default()
    }
}

/// Disharmony — steals an attacker mid-swing.
pub fn disharmony() -> CardDefinition {
    let attacker = || Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::IsAttacking) };
    CardDefinition {
        name: "Disharmony",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        cast_only_before_blockers: true,
        effect: Effect::Seq(vec![
            Effect::Untap { what: attacker(), up_to: None },
            Effect::RemoveFromCombat { what: target() },
            Effect::GainControl {
                what: target(),
                to: None,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}


// ── Wave 7b ────────────────────────────────────────────────────────────────

/// Ayesha Tanaka — a white tax on every artifact's activated ability.
pub fn ayesha_tanaka() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Banding],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CounterUnless {
                what: Selector::TargetFiltered { slot: 0, filter: R::Artifact },
                cost: crate::card::WardCost::Mana(cost(&[w()])),
            },
            ..Default::default()
        }],
        ..legend(
            "Ayesha Tanaka",
            cost(&[w(), w(), u(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// Cocoon — three turns of sleep, then a flying upgrade. The pupa counters
/// ride the enchanted creature so the untap lock reads them directly.
pub fn cocoon() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap while it has a pupa counter.",
            effect: StaticEffect::GrantKeyword {
                applies_to: host(),
                keyword: Keyword::DoesntUntapWhileCounter(CounterType::Pupa),
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Tap { what: host() },
                    Effect::AddCounter {
                        what: host(),
                        kind: CounterType::Pupa,
                        amount: Value::Const(3),
                    },
                ]),
            },
            TriggeredAbility {
                event: upkeep(EventScope::YourControl),
                effect: Effect::If {
                    cond: Predicate::EntityMatches {
                        what: host(),
                        filter: R::WithCounterAtLeast(CounterType::Pupa, 1),
                    },
                    then: Box::new(Effect::RemoveCounter {
                        what: host(),
                        kind: CounterType::Pupa,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: host(),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(1),
                        },
                        Effect::GrantKeyword {
                            what: host(),
                            keyword: Keyword::Flying,
                            duration: Duration::Permanent,
                        },
                        Effect::SacrificeSource,
                    ])),
                },
            },
        ],
        ..aura("Cocoon", cost(&[g()]), R::Creature.and(R::ControlledByYou))
    }
}

/// Rasputin Dreamweaver — seven dreams of mana or prevention, refilled while
/// he stays untapped.
pub fn rasputin_dreamweaver() -> CardDefinition {
    let spend = |effect: Effect| ActivatedAbility {
        remove_counter_cost: Some((CounterType::Dream, 1)),
        effect,
        ..Default::default()
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::Dream, Value::Const(7))),
        activated_abilities: vec![
            spend(Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Colorless(Value::Const(1)),
            }),
            spend(Effect::PreventNextDamage {
                target: Selector::This,
                amount: Value::Const(1),
            }),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::This, filter: R::Untapped },
                // The printed cap: he never holds more than seven.
                Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::WithCounterAtLeast(CounterType::Dream, 7),
                })),
            ])),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Dream,
                amount: Value::Const(1),
            },
        }],
        ..legend(
            "Rasputin Dreamweaver",
            cost(&[generic(4), w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            1,
        )
    }
}

/// Voodoo Doll — it hurts more every turn, and it hurts you if you forget it.
pub fn voodoo_doll() -> CardDefinition {
    let pins = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Pin };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: upkeep(EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Pin,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::End),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Untapped,
                }),
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::You),
                        amount: pins(),
                    },
                    Effect::Destroy { what: Selector::This },
                ]),
            },
        ],
        ..artifact(
            "Voodoo Doll",
            cost(&[generic(6)]),
            vec![ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[x(), x()]),
                // X is the pin count, so the ability costs twice what it deals.
                condition: Some(Predicate::ValueEquals(Value::XFromCost, pins())),
                effect: Effect::DealDamage { to: target_any(), amount: pins() },
                ..Default::default()
            }],
        )
    }
}

/// Johan — trade his attack for a free swing from everyone else.
pub fn johan() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Johan can't attack; your creatures don't tap to attack".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::CantAttack,
                        duration: Duration::EndOfCombat,
                    },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        keyword: Keyword::Vigilance,
                        duration: Duration::EndOfCombat,
                    },
                ])),
            },
        }],
        ..legend(
            "Johan",
            cost(&[generic(3), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            5,
            4,
        )
    }
}

/// Gabriel Angelfire — picks a new combat trick every upkeep.
pub fn gabriel_angelfire() -> CardDefinition {
    let gain = |kw: Keyword| Effect::GrantKeyword {
        what: Selector::This,
        keyword: kw,
        duration: Duration::UntilYourNextUntap,
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::ChooseN {
                picks: vec![0],
                modes: vec![
                    gain(Keyword::Flying),
                    gain(Keyword::FirstStrike),
                    gain(Keyword::Trample),
                    gain(Keyword::Rampage(3)),
                ],
            },
        }],
        ..legend(
            "Gabriel Angelfire",
            cost(&[generic(3), g(), g(), w(), w()]),
            vec![CreatureType::Angel],
            4,
            4,
        )
    }
}

/// Nova Pentacle — hand the next hit to one of their own creatures.
pub fn nova_pentacle() -> CardDefinition {
    artifact(
        "Nova Pentacle",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: Some(Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByOpponent),
                }),
                whole_turn: false,
            },
            ..Default::default()
        }],
    )
}

/// Puppet Master — buy the dead creature back, and yourself too if you pay.
pub fn puppet_master() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                },
                Effect::MayPay {
                    description: "Return Puppet Master to your hand".into(),
                    mana_cost: cost(&[u(), u(), u()]),
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: None,
                },
            ]),
        }],
        ..aura("Puppet Master", cost(&[u(), u(), u()]), R::Creature)
    }
}

/// Relic Bind — every tap of their artifact costs them a point, or buys one.
pub fn relic_bind() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsHostOfSource },
            ),
            effect: Effect::ChooseN {
                picks: vec![0],
                modes: vec![
                    Effect::DealDamage {
                        to: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::Player.or(R::Planeswalker),
                        },
                        amount: Value::Const(1),
                    },
                    Effect::GainLife {
                        who: Selector::TargetFiltered { slot: 0, filter: R::Player },
                        amount: Value::Const(1),
                    },
                ],
            },
        }],
        ..aura("Relic Bind", cost(&[generic(2), u()]), R::Artifact.and(R::ControlledByOpponent))
    }
}

/// Floral Spuzzem — smashes an artifact instead of connecting.
pub fn floral_spuzzem() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Destroy target artifact defending player controls".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Destroy {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::Artifact.and(R::ControlledByOpponent),
                        },
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::DealsNoCombatDamage,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        }],
        ..creature("Floral Spuzzem", cost(&[generic(3), g()]), vec![CreatureType::Elemental], 2, 2)
    }
}

/// Falling Star — a dexterity card; the engine settles for one random hit.
pub fn falling_star() -> CardDefinition {
    CardDefinition {
        name: "Falling Star",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::RandomAmong(R::Creature),
                amount: Value::Const(3),
            },
            Effect::Tap { what: Selector::DamagedThisResolution { filter: R::Creature } },
        ]),
        ..Default::default()
    }
}

/// Remove Enchantments — take yours back, sweep the rest away.
pub fn remove_enchantments() -> CardDefinition {
    let mine = || R::Enchantment.and(R::ControlledByYou);
    CardDefinition {
        name: "Remove Enchantments",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::EachPermanent(mine().and(R::OwnedByYou)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Destroy { what: Selector::EachPermanent(mine()) },
        ]),
        ..Default::default()
    }
}
