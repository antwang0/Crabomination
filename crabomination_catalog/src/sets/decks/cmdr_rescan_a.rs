//! Commander: one-card gaps of the 2026-10-14 ranks 1-5500 EDHREC rescan —
//! each card here is the last one a seated average deck was missing. Tests
//! in `tests/recent_b/cmdr_rescan_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, ConditionalEquipBonus,
    CounterType, CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{add_colorless, add_mana, draw, lose_life, mint_token, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn your_step(step: TurnStep) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl)
}

fn your_creatures() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou))
}

/// Pump `what` +`p`/+0 and grant trample until end of turn.
fn rampage(what: Selector, p: i32) -> Effect {
    Effect::Seq(vec![
        Effect::PumpPT {
            what: what.clone(),
            power: Value::Const(p),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        Effect::GrantKeyword { what, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
    ])
}

/// Invigorated Rampage — {1}{R} instant. Choose one — target creature gets
/// +4/+0 and gains trample until end of turn; or two target creatures each
/// get +2/+0 and gain trample until end of turn. (Cleon, Seifer.)
pub fn invigorated_rampage() -> CardDefinition {
    CardDefinition {
        name: "Invigorated Rampage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            rampage(target_filtered(R::Creature), 4),
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Creature,
                effect: Box::new(rampage(Selector::Target(0), 2)),
            },
        ]),
        ..Default::default()
    }
}

/// Lace with Moonglove — {2}{G} instant. Target creature gains deathtouch
/// until end of turn. Draw a card. (Asmira.)
pub fn lace_with_moonglove() -> CardDefinition {
    CardDefinition {
        name: "Lace with Moonglove",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Swarm of Bloodflies — {4}{B} 0/0 Insect; flying. Enters with two +1/+1
/// counters; whenever another creature dies, a +1/+1 counter on it. (Fumulus.)
pub fn swarm_of_bloodflies() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf))),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Swarm of Bloodflies", cost(&[generic(4), b()]), vec![CreatureType::Insect], 0, 0)
    }
}

/// Bloodcrazed Paladin — {1}{B} 1/1 Vampire Knight; flash. Enters with a
/// +1/+1 counter for each creature that died this turn. (Massacre Girl.)
pub fn bloodcrazed_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::CreaturesDiedThisTurnTotal)),
        ..creature(
            "Bloodcrazed Paladin",
            cost(&[generic(1), b()]),
            vec![CreatureType::Vampire, CreatureType::Knight],
            1,
            1,
        )
    }
}

/// Homicidal Seclusion — {4}{B} enchantment. As long as you control exactly
/// one creature, that creature gets +3/+1 and has lifelink. (Shauku.)
pub fn homicidal_seclusion() -> CardDefinition {
    CardDefinition {
        name: "Homicidal Seclusion",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "As long as you control exactly one creature, that creature gets +3/+1 and has lifelink.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::ValueEquals(Value::count(your_creatures()), Value::ONE),
                applies_to: your_creatures(),
                power: 3,
                toughness: 1,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// The Scarlet Witch — {2}{R} legendary 2/3 Mutant Warlock Hero. Instant and
/// sorcery spells you cast with mana value 4 or greater cost {X} less, X her
/// power. (Scarlet Witch, Chaotic Avenger; The Vision and Scarlet Witch.)
pub fn the_scarlet_witch() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast with mana value 4 or greater cost {X} less, X = her power.",
            effect: StaticEffect::CostReductionBySourcePower {
                filter: R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::ManaValueAtLeast(4)),
            },
        }],
        ..legend(
            "The Scarlet Witch",
            cost(&[generic(2), r()]),
            vec![CreatureType::Mutant, CreatureType::Warlock, CreatureType::Hero],
            2,
            3,
        )
    }
}

/// Gnoll Hunter — {1}{G} 2/2 Gnoll. Pack tactics — whenever it attacks, if
/// you attacked with total power 6 or greater, a +1/+1 counter on it. (Targ
/// Nar.)
pub fn gnoll_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithTotalPowerAtLeast { who: PlayerRef::You, at_least: 6 },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Gnoll Hunter", cost(&[generic(1), g()]), vec![CreatureType::Gnoll], 2, 2)
    }
}

/// Dragonscale General — {3}{W} 2/3 Human Warrior. At the beginning of your
/// end step, bolster X, X the tapped creatures you control. (Anafenza,
/// Kin-Tree Spirit.)
pub fn dragonscale_general() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: Effect::AddCounter {
                what: Selector::LeastToughnessYouControl,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::Tapped),
                )),
            },
        }],
        ..creature(
            "Dragonscale General",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Mutant Chain Reaction — {2}{G} sorcery. Destroy up to one target artifact,
/// enchantment, or creature with flying. Create a Mutagen token. (Mutagen
/// Man.)
pub fn mutant_chain_reaction() -> CardDefinition {
    CardDefinition {
        name: "Mutant Chain Reaction",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Artifact
                    .or(R::Enchantment)
                    .or(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
            mint_token(crabomination_base::tokens::mutagen_token(), 1),
        ]),
        ..Default::default()
    }
}

/// Gargantos, the Endbringer — {2}{B}{G} legendary 0/0 Elder God Horror;
/// deathtouch. Enters with three +1/+1 counters; at the beginning of your
/// upkeep, double them. Whenever he deals combat damage to an opponent, he
/// deals that much damage to each other opponent.
pub fn gargantos_the_endbringer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_step(TurnStep::Upkeep),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..legend(
            "Gargantos, the Endbringer",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Elder, CreatureType::God, CreatureType::Horror],
            0,
            0,
        )
    }
}

/// Davvol, Evincar of Rath — {2}{B}{G} legendary 2/2 Phyrexian Noble.
/// Whenever another creature you control enters, you lose 1 life and add {B}.
pub fn davvol_evincar_of_rath() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::Seq(vec![lose_life(1, Selector::You), add_mana(vec![Color::Black])]),
        }],
        ..legend(
            "Davvol, Evincar of Rath",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Noble],
            2,
            2,
        )
    }
}

/// Kinbinding — {3}{W}{W} enchantment. Creatures you control get +X/+X, X
/// the creatures that entered under your control this turn. At the beginning
/// of combat on your turn, create a 1/1 green and white Kithkin. (Sally
/// Pride.)
pub fn kinbinding() -> CardDefinition {
    let x = Value::CreaturesEnteredThisTurn(PlayerRef::You);
    CardDefinition {
        name: "Kinbinding",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +X/+X, where X is the number of creatures that entered the battlefield under your control this turn.",
            effect: StaticEffect::PumpPTByValue { applies_to: your_creatures(), power: x.clone(), toughness: x },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::BeginCombat),
            effect: mint_token(
                TokenDefinition {
                    name: "Kithkin".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green, Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Kithkin], ..Default::default() },
                    ..Default::default()
                },
                1,
            ),
        }],
        ..Default::default()
    }
}

/// Gimli's Axe — {2}{R} Equipment. Equipped creature gets +3/+0 and, while
/// it is legendary, has menace. Equip {2}. (Gimli of the Glittering Caves.)
pub fn gimlis_axe() -> CardDefinition {
    CardDefinition {
        name: "Gimli's Axe",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::HasSupertype(Supertype::Legendary),
                keywords: vec![Keyword::Menace],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Weather Maker — {3} artifact. Landfall: a charge counter on it. {T}: Add
/// one mana of any color. {T}, remove two charge counters: Add {C}{C}. {T},
/// remove three: it deals 3 damage to any target. (Gandalf, Shadow's Foe.)
pub fn weather_maker() -> CardDefinition {
    CardDefinition {
        name: "Weather Maker",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::effect::shortcut::landfall(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Charge,
            amount: Value::ONE,
        })],
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 2)),
                effect: add_colorless(2),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 3)),
                effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hydro-Man, Fluid Felon — {U}{U} legendary 2/2 Elemental Villain. Whenever
/// you cast a blue spell, if he's a creature, +1/+1 until end of turn. At the
/// beginning of your end step, untap him; until your next turn he becomes a
/// land with "{T}: Add {U}" and isn't a creature. (The Blue Spirit,
/// Hydro-Man, Blue Marvel.) The granted mana ability is printed here, live
/// only while he is a land.
pub fn hydro_man_fluid_felon() -> CardDefinition {
    let this_is = |filter| Predicate::EntityMatches { what: Selector::This, filter };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: add_mana(vec![Color::Blue]),
            condition: Some(this_is(R::Land)),
            ..Default::default()
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::HasColor(Color::Blue),
                        },
                        this_is(R::Creature),
                    ]),
                ),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: your_step(TurnStep::End),
                effect: Effect::Seq(vec![
                    Effect::Untap { what: Selector::This, up_to: None },
                    Effect::SetCardTypesFor {
                        what: Selector::This,
                        card_types: vec![CardType::Land],
                        duration: Duration::UntilNextTurn,
                    },
                ]),
            },
        ],
        ..legend(
            "Hydro-Man, Fluid Felon",
            cost(&[u(), u()]),
            vec![CreatureType::Elemental, CreatureType::Villain],
            2,
            2,
        )
    }
}

/// Khalni Hydra — {G}{G}{G}{G}{G}{G}{G}{G} 8/8 Hydra; trample. Costs {G}
/// less for each green creature you control. (Renata, Nylea.)
pub fn khalni_hydra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {G} less to cast for each green creature you control.",
            effect: StaticEffect::SelfColoredCostReducedPer {
                less: cost(&[g()]),
                count: Value::count(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::HasColor(Color::Green)),
                )),
            },
        }],
        ..creature("Khalni Hydra", cost(&[g(), g(), g(), g(), g(), g(), g(), g()]), vec![CreatureType::Hydra], 8, 8)
    }
}

/// Hall of Echoes — land. {T}: Add {C}. {5}: it becomes a copy of target
/// creature you control until end of turn, and the legend rule doesn't apply
/// to permanents you control this turn. (Shisato, Kei Takahashi, Machine
/// Man.)
pub fn hall_of_echoes() -> CardDefinition {
    CardDefinition {
        name: "Hall of Echoes",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                effect: Effect::Seq(vec![
                    Effect::BecomeCopyOfFor {
                        what: Selector::This,
                        source: target_filtered(R::Creature.and(R::ControlledByYou)),
                        duration: Duration::EndOfTurn,
                        non_legendary: false,
                    },
                    Effect::LegendRuleOffThisTurn { who: PlayerRef::You },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
