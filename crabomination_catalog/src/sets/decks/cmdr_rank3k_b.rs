//! Commander: one-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), second file. Tests in `tests/recent_b/cmdr_rank3k_b.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, Adventure, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn samurai_or_warrior_attacks_alone(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::All(vec![
            Predicate::AttackingAlone,
            trigger_is(R::HasCreatureType(CreatureType::Samurai).or(R::HasCreatureType(CreatureType::Warrior))),
        ])),
        effect,
    }
}

/// Demonic Vigor — {B} Aura: +1/+1; when the enchanted creature dies, return
/// that card to its owner's hand.
pub fn demonic_vigor() -> CardDefinition {
    aura(
        "Demonic Vigor",
        cost(&[b()]),
        R::Creature,
        EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
            }],
            ..Default::default()
        },
    )
}

/// Stark's Ingenuity — {2}{U} Aura (creature you control). ETB: you may pay
/// {X} to draw X. Whenever you draw a card, a +1/+1 counter on the enchanted
/// creature.
pub fn starks_ingenuity() -> CardDefinition {
    CardDefinition {
        attach_only_filter: Some(R::Creature.and(R::ControlledByYou)),
        triggered_abilities: vec![
            etb(Effect::MayPayX {
                description: "Pay {X} to draw X cards?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::XFromCost }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::AddCounter { what: host(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..aura(
            "Stark's Ingenuity",
            cost(&[generic(2), u()]),
            R::Creature.and(R::ControlledByYou),
            EquipBonus::default(),
        )
    }
}

/// Seismic Tutelage — {3}{G} Aura. ETB: a +1/+1 counter on the enchanted
/// creature; whenever it attacks, double its +1/+1 counters.
pub fn seismic_tutelage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: host(),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..aura(
            "Seismic Tutelage",
            cost(&[generic(3), g()]),
            R::Creature,
            EquipBonus {
                triggered_abilities: vec![on_attack(Effect::DoubleCountersOnEach {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                })],
                ..Default::default()
            },
        )
    }
}

/// Xiahou Dun, the One-Eyed — {2}{B}{B} legendary 3/2, horsemanship. Sacrifice
/// it: return target black card from your graveyard to your hand, only during
/// your turn before attackers are declared.
pub fn xiahou_dun_the_one_eyed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Any(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::CurrentStepIs(TurnStep::Draw),
                    Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                    Predicate::CurrentStepIs(TurnStep::BeginCombat),
                ]),
            ])),
            effect: Effect::Move {
                what: target_filtered(R::HasColor(Color::Black).and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..legend(
            "Xiahou Dun, the One-Eyed",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            2,
        )
    }
}

/// Arni, Humble Scribe — {2}{U} legendary 3/2. Another nontoken creature you
/// control enters: untap Arni. {T}: draw, then discard.
pub fn arni_humble_scribe() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(trigger_is(R::Creature.and(R::NotToken))),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Arni, Humble Scribe",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Imperial Subduer — {2}{W} 3/2 Human Samurai. Whenever a Samurai or Warrior
/// you control attacks alone, tap target creature you don't control.
pub fn imperial_subduer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![samurai_or_warrior_attacks_alone(Effect::Tap {
            what: target_filtered(R::Creature.and(R::ControlledByYou.negate())),
        })],
        ..creature("Imperial Subduer", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Samurai], 3, 2)
    }
}

/// Peerless Samurai — {2}{R} 2/3 Human Samurai, menace. Whenever a Samurai or
/// Warrior you control attacks alone, the next spell you cast this turn costs
/// {1} less.
pub fn peerless_samurai() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![samurai_or_warrior_attacks_alone(Effect::GrantNextSpellDiscountThisTurn {
            amount: Value::ONE,
        })],
        ..creature("Peerless Samurai", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Samurai], 2, 3)
    }
}

/// Campfire — {1} artifact. {1}, {T}: gain 2 life. {2}, {T}, exile it: put all
/// commanders you own from the command zone and your graveyard into your hand,
/// then shuffle your graveyard into your library.
pub fn campfire() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                exile_self_cost: true,
                effect: Effect::Seq(vec![
                    // `CommanderToHand` moves one commander a call; a partner
                    // pair takes the second.
                    Effect::CommanderToHand { who: PlayerRef::You },
                    Effect::CommanderToHand { who: PlayerRef::You },
                    Effect::Move {
                        what: Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::IsCommander.and(R::OwnedByYou),
                        },
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                    Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::You },
                ]),
                ..Default::default()
            },
        ],
        ..spell("Campfire", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Vogar, Necropolis Tyrant — {3}{B}{B} legendary 4/4 menace. Another creature
/// dies during your turn: a +1/+1 counter. Dies: draw a card per +1/+1 counter.
pub fn vogar_necropolis_tyrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    trigger_is(R::OtherThanSource),
                ])),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            on_dies(Effect::Draw {
                who: Selector::You,
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
            }),
        ],
        ..legend(
            "Vogar, Necropolis Tyrant",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Giant],
            4,
            4,
        )
    }
}

/// Herald of Ilharg — {2}{R}{G} 3/3 trample. Whenever you cast a creature
/// spell, two +1/+1 counters on it; if that spell has mana value 5 or more, it
/// deals damage equal to its counters to each opponent.
pub fn herald_of_ilharg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(trigger_is(R::HasCardType(CardType::Creature))),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::If {
                    cond: trigger_is(R::ManaValueAtLeast(5)),
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature("Herald of Ilharg", cost(&[generic(2), r(), g()]), vec![CreatureType::Boar, CreatureType::Beast], 3, 3)
    }
}

/// Force of Virtue — {2}{W}{W} enchantment, flash. Off your turn, exile a white
/// card from your hand instead of paying. Creatures you control get +1/+1.
pub fn force_of_virtue() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            exile_filter: Some(R::HasColor(Color::White)),
            not_your_turn_only: true,
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 1,
                toughness: 1,
            },
        }],
        ..spell("Force of Virtue", cost(&[generic(2), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Vision of Love — {1}{R} instant: you may sacrifice an artifact or discard a
/// card; if you do, draw two cards.
pub fn vision_of_love() -> CardDefinition {
    let draw_two = || Effect::Draw { who: Selector::You, amount: Value::Const(2) };
    spell(
        "Vision of Love",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::MaySacrifice {
            description: "Sacrifice an artifact to draw two?".into(),
            filter: R::Artifact,
            count: Value::ONE,
            then: Box::new(draw_two()),
            else_: Some(Box::new(Effect::MayDiscard {
                description: "Discard a card to draw two?".into(),
                count: Value::ONE,
                then: Box::new(draw_two()),
                else_: None,
            })),
        },
    )
}

/// Ultron the Annihilator — {3}{B}{B} legendary 3/4 flying artifact creature.
/// ETB or attack: a 2/2 colorless Robot Villain artifact creature. Whenever
/// another artifact card is put into your graveyard, each opponent loses 1.
pub fn ultron_the_annihilator() -> CardDefinition {
    let robot = Arc::new(TokenDefinition {
        name: "Robot Villain".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot, CreatureType::Villain], ..Default::default() },
        ..Default::default()
    });
    let make = || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: robot.clone() };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(make()),
            on_attack(make()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl)
                    .with_filter(trigger_is(R::Artifact.and(R::OtherThanSource))),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            },
        ],
        ..legend(
            "Ultron the Annihilator",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Robot, CreatureType::Villain],
            3,
            4,
        )
    }
}

/// Nadier, Agent of the Duskenel — {5}{B} legendary 3/3, partner. Whenever a
/// token you control leaves the battlefield, a +1/+1 counter on Nadier. When
/// Nadier leaves, 1/1 green Elf Warriors equal to its power.
pub fn nadier_agent_of_the_duskenel() -> CardDefinition {
    let elf = Arc::new(TokenDefinition {
        name: "Elf Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                    .with_filter(trigger_is(R::IsToken)),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::PowerOf(Box::new(Selector::This)),
                    definition: elf,
                },
            },
        ],
        ..legend(
            "Nadier, Agent of the Duskenel",
            cost(&[generic(5), b()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Molten Monstrosity — {7}{R} 5/5 Hellion, trample; costs {X} less, X the
/// greatest power among creatures you control.
pub fn molten_monstrosity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the greatest power among creatures you control.",
            effect: StaticEffect::SelfCostReducedByGreatestPower,
        }],
        ..creature("Molten Monstrosity", cost(&[generic(7), r()]), vec![CreatureType::Hellion], 5, 5)
    }
}

/// Leafkin Avenger — {2}{R}{G} 4/3 Elemental Druid. {T}: add {G} for each
/// creature with power 4 or greater you control. {7}{R}: damage equal to its
/// power to target player or planeswalker.
pub fn leafkin_avenger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(
                        Color::Green,
                        Value::count(Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                        )),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(7), r()]),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Leafkin Avenger",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Elemental, CreatureType::Druid],
            4,
            3,
        )
    }
}

/// Guardian Naga // Banishing Coils — {5}{W}{W} 5/6 vigilant Snake; during your
/// turn, prevent all damage to it. Adventure {2}{W} instant: exile target
/// artifact or enchantment.
pub fn guardian_naga() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "During your turn, prevent all damage that would be dealt to this creature.",
            effect: StaticEffect::WhileYourTurn { inner: Box::new(StaticEffect::PreventAllDamageToThis) },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Banishing Coils",
            cost: cost(&[generic(2), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Exile { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        })),
        ..creature("Guardian Naga", cost(&[generic(5), w(), w()]), vec![CreatureType::Snake], 5, 6)
    }
}

/// Dread Linnorm // Scale Deflection — {6}{G} 7/6 Snake Dragon; can't be
/// blocked by power 3 or less. Adventure {3}{G} instant: two +1/+1 counters on
/// target creature, untap it, hexproof until end of turn.
pub fn dread_linnorm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerAtMost(3)],
        adventure: Some(Box::new(Adventure {
            name: "Scale Deflection",
            cost: cost(&[generic(3), g()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
            ]),
        })),
        ..creature("Dread Linnorm", cost(&[generic(6), g()]), vec![CreatureType::Snake, CreatureType::Dragon], 7, 6)
    }
}

/// Omnispell Adept — {4}{U} 3/4 Human Wizard. {2}{U}, {T}: you may cast an
/// instant or sorcery spell from your hand without paying its mana cost.
pub fn omnispell_adept() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            effect: Effect::MayCastFromHandFreeMatching {
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                max_mv: Value::Const(99),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..creature("Omnispell Adept", cost(&[generic(4), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 4)
    }
}

/// Orca, Siege Demon — {5}{B}{R} legendary 5/5 trample. Another creature dies:
/// a +1/+1 counter. Dies: damage equal to its power divided among any number
/// of targets.
pub fn orca_siege_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(trigger_is(R::OtherThanSource)),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            on_dies(Effect::DealDamageDivided {
                total: Value::PowerOf(Box::new(Selector::This)),
                filter: R::Any,
                max_targets: 4,
                retaliate_to_source: false,
            }),
        ],
        ..legend("Orca, Siege Demon", cost(&[generic(5), b(), r()]), vec![CreatureType::Demon], 5, 5)
    }
}

/// Desperate Plea — {1}{B} sorcery — Lesson; sacrifice a creature as an
/// additional cost. Choose one or both: return target creature card from your
/// graveyard to the battlefield if its power is at most the sacrificed
/// creature's; destroy target creature.
pub fn desperate_plea() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![crate::card::SpellSubtype::Lesson], ..Default::default() },
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 1 }],
        effect: Effect::ChooseModesCast {
            min: 1,
            max: 2,
            allow_repeats: false,
            modes: vec![
                // "…if its power is less than or equal to the sacrificed
                // creature's power" is checked on resolution, not a target
                // restriction.
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::SacrificedPower, Value::PowerOf(Box::new(Selector::Target(0)))),
                    then: Box::new(Effect::Move {
                        what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Destroy { what: target_filtered(R::Creature) },
            ],
        },
        ..spell("Desperate Plea", cost(&[generic(1), b()]), CardType::Sorcery, Effect::Noop)
    }
}

/// Danitha, Spear of Agony — {2}{B} legendary 2/2 first strike. Whenever you
/// cast a spell that targets an opponent or a creature an opponent controls, a
/// +1/+1 counter on Danitha.
pub fn danitha_spear_of_agony() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellTargetsMatch(
                    R::OpponentPlayer.or(R::Creature.and(R::ControlledByOpponent)),
                )),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend(
            "Danitha, Spear of Agony",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Sutina, Speaker of the Tajuru — {2}{G} legendary 2/2. ETB: a basic land
/// onto the battlefield tapped. Attacks: you may return a land you control to
/// its owner's hand; when you do, a +1/+1 counter on target creature.
pub fn sutina_speaker_of_the_tajuru() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
            on_attack(Effect::MayDo {
                description: "Return a land you control to its owner's hand?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::ReturnOneYouControl { filter: R::Land, keep_best: false },
                    Effect::Reflexive {
                        body: Box::new(Effect::AddCounter {
                            what: target_filtered(R::Creature),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    },
                ])),
            }),
        ],
        ..legend(
            "Sutina, Speaker of the Tajuru",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Spider-Man, New Champion — {2}{R} legendary 3/3 reach. Whenever you discard
/// one or more cards, that much damage to any target.
pub fn spider_man_new_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DiscardedOneOrMore, EventScope::YourControl),
            effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::TriggerEventAmount },
        }],
        ..legend(
            "Spider-Man, New Champion",
            cost(&[generic(2), r()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            3,
            3,
        )
    }
}

/// The Destined White Mage — {2}{W} legendary 2/3 lifelink.
/// {W}, {T}: another target creature you control gains lifelink until end of
/// turn. Whenever you gain life, a +1/+1 counter on target creature you
/// control — three with a full party.
pub fn the_destined_white_mage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::PartyCount, Value::Const(4)),
                then: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..legend(
            "The Destined White Mage",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Jet, Rebel Leader — {3}{W} legendary 3/4. Attacks: look at the
/// top five; you may put a creature card with mana value 3 or less onto the
/// battlefield tapped and attacking; the rest on the bottom in a random order.
pub fn jet_rebel_leader() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::LookTopMayDeployAttacking {
            count: Value::Const(5),
            filter: R::Creature.and(R::ManaValueAtMost(3)),
            return_at_end_of_combat: false,
            plain: true,
        })],
        ..legend(
            "Jet, Rebel Leader",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            3,
            4,
        )
    }
}
