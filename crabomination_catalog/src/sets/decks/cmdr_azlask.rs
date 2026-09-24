//! Commander: the cards the **Eldrazi Incursion** precon (M3C, Azlask, the
//! Swelling Scourge) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_azlask.rs`.
//!
//! Residuals (each also on its card):
//! - **Benthic Anomaly** — each opponent's greatest-power creature is chosen,
//!   and the copy is of the one with the greatest mana value.
//! - **Bismuth Mindrender** — the exiled card may be cast for life until end
//!   of turn, not only as the trigger resolves.
//! - **Selective Obliteration** — each player's color is the one most common
//!   among their permanents.
//! - **Twins of Discord** — the granted bloodthirst rides colorless creature
//!   spells you cast (as Bloodlord of Vaasgoth), not every entry.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::shortcut::{etb, myriad, on_cast, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{colorless, colorless_hybrid, cost, g, generic, u, w, Color, ManaCost};
use crabomination_base::tokens::{eldrazi_scion_token, eldrazi_spawn_token};
use std::sync::Arc;

fn eldrazi(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn devoid(mut def: CardDefinition) -> CardDefinition {
    def.keywords.insert(0, Keyword::Devoid);
    def
}

fn spawns(n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(eldrazi_spawn_token()) }
}

fn colorless_obj() -> R {
    R::Colorless
}

/// Azlask, the Swelling Scourge — colorless deaths give experience; WUBRG
/// pumps the team by it and arms Scions and Spawns.
pub fn azlask_the_swelling_scourge() -> CardDefinition {
    let x = || Value::ControllerExperience;
    let scions_spawns =
        || R::HasCreatureType(CreatureType::Scion).or(R::HasCreatureType(CreatureType::Spawn)).and(R::ControlledByYou);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: colorless_obj() },
            ),
            effect: Effect::AddExperience(Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), crate::mana::b(), crate::mana::r(), g()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: x(),
                    toughness: x(),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeywords {
                    what: Selector::EachPermanent(scions_spawns()),
                    keywords: vec![Keyword::Indestructible, Keyword::Annihilator(1)],
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..eldrazi("Azlask, the Swelling Scourge", cost(&[generic(3)]), 2, 2)
    }
}

/// Angelic Aberration — sacrifice any number of 1-power-or-toughness
/// creatures for 4/4 flying, vigilant Eldrazi Angels.
pub fn angelic_aberration() -> CardDefinition {
    let angel = TokenDefinition {
        name: "Eldrazi Angel".into(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi, CreatureType::Angel], ..Default::default() },
        ..Default::default()
    };
    let mut def = devoid(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Creature.and(R::BasePowerOrToughnessAtMost(1)),
            per_each: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(angel),
            }),
        })],
        ..eldrazi("Angelic Aberration", cost(&[generic(5), w()]), 4, 4)
    });
    def.subtypes.creature_types.push(CreatureType::Angel);
    def
}

/// Benthic Anomaly — on cast, a summed copy of one creature per opponent.
/// Residual: the choices are the engine's.
pub fn benthic_anomaly() -> CardDefinition {
    let mut def = devoid(CardDefinition {
        triggered_abilities: vec![on_cast(Effect::CopyOnePerOpponentWithTotalStats)],
        ..eldrazi("Benthic Anomaly", cost(&[generic(6), u()]), 7, 8)
    });
    def.subtypes.creature_types.push(CreatureType::Serpent);
    def
}

/// Bismuth Mindrender — menace; its hit flips the player's next nonland card
/// for you to cast for life. Residual: castable until end of turn.
pub fn bismuth_mindrender() -> CardDefinition {
    devoid(CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileTopUntilNonland { who: PlayerRef::TriggerEventPlayer },
                Effect::GrantMayPlayForLife {
                    what: Selector::ExiledThisResolution { filter: R::Nonland },
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                },
            ]),
        }],
        ..eldrazi("Bismuth Mindrender", cost(&[generic(3), crate::mana::b()]), 4, 3)
    })
}

/// Chittering Dispatcher — myriad; leaving the battlefield leaves a Spawn.
pub fn chittering_dispatcher() -> CardDefinition {
    let mut def = devoid(CardDefinition {
        triggered_abilities: vec![
            myriad(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: spawns(1),
            },
        ],
        ..eldrazi("Chittering Dispatcher", cost(&[generic(2), g()]), 2, 3)
    });
    def.subtypes.creature_types.push(CreatureType::Drone);
    def
}

fn kindred_eldrazi() -> Subtypes {
    Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() }
}

/// Eldrazi Conscription — +10/+10, trample and annihilator 2.
pub fn eldrazi_conscription() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Eldrazi Conscription",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..kindred_eldrazi() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +10/+10.",
                effect: StaticEffect::PumpPT { applies_to: host(), power: 10, toughness: 10 },
            },
            StaticAbility {
                description: "Enchanted creature has trample.",
                effect: StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::Trample },
            },
            StaticAbility {
                description: "Enchanted creature has annihilator 2.",
                effect: StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::Annihilator(2) },
            },
        ],
        ..Default::default()
    }
}

/// Eldritch Immunity — protection from each color for one creature of yours,
/// or all of them overloaded.
pub fn eldritch_immunity() -> CardDefinition {
    let protect = |what: Selector| Effect::GrantKeywords {
        what,
        keywords: Color::ALL.iter().map(|c| Keyword::Protection(*c)).collect(),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Eldritch Immunity",
        cost: cost(&[colorless(1)]),
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: kindred_eldrazi(),
        effect: protect(target_filtered(R::Creature.and(R::ControlledByYou))),
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(4), colorless(1)]),
            effect_override: Some(protect(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)))),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Hideous Taskmaster — on cast, borrows a creature from each opponent until
/// end of turn, untapped and armed.
pub fn hideous_taskmaster() -> CardDefinition {
    devoid(CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Annihilator(1)],
        triggered_abilities: vec![on_cast(Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 5,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Seq(vec![
                    Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::EndOfTurn },
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                    Effect::GrantKeywords {
                        what: Selector::Target(0),
                        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Annihilator(1)],
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
        })],
        ..eldrazi("Hideous Taskmaster", cost(&[generic(6), crate::mana::r()]), 7, 2)
    })
}

/// Inversion Behemoth — each combat on your turn, switch any number of
/// creatures' power and toughness.
pub fn inversion_behemoth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::SwitchPT { what: Selector::Target(0), duration: Duration::EndOfTurn }),
            },
        }],
        ..eldrazi("Inversion Behemoth", cost(&[generic(2), colorless(1), colorless(1)]), 2, 9)
    }
}

/// Morophon, the Boundless — changeling; spells of the chosen type cost WUBRG
/// less; others of that type get +1/+1.
pub fn morophon_the_boundless() -> CardDefinition {
    CardDefinition {
        name: "Morophon, the Boundless",
        cost: cost(&[generic(7)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Changeling],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![
            StaticAbility {
                description: "Spells of the chosen type you cast cost {W}{U}{B}{R}{G} less.",
                effect: StaticEffect::ColoredCostReduction {
                    filter: R::IsSourceChosenCreatureType,
                    less: cost(&[w(), u(), crate::mana::b(), crate::mana::r(), g()]),
                },
            },
            StaticAbility {
                description: "Other creatures you control of the chosen type get +1/+1.",
                effect: StaticEffect::AnthemForChosenType {
                    power: 1,
                    toughness: 1,
                    exclude_source: true,
                    opponents: false,
                    all_players: false,
                    per_counter: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Selective Obliteration — each player keeps only one color. Residual: the
/// colors are the engine's.
pub fn selective_obliteration() -> CardDefinition {
    CardDefinition {
        name: "Selective Obliteration",
        cost: cost(&[generic(3), colorless(1), colorless(1)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerChoosesColorExileOthers,
        ..Default::default()
    }
}

/// Skittering Invasion — five Eldrazi Spawn.
pub fn skittering_invasion() -> CardDefinition {
    CardDefinition {
        name: "Skittering Invasion",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: kindred_eldrazi(),
        effect: spawns(5),
        ..Default::default()
    }
}

/// Spawnbed Protector — each end step, an Eldrazi back to hand and two Scions.
pub fn spawnbed_protector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: target_filtered(
                            R::Creature.and(R::HasCreatureType(CreatureType::Eldrazi)).and(R::InYourGraveyard),
                        ),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: Arc::new(eldrazi_scion_token()),
                },
            ]),
        }],
        ..eldrazi("Spawnbed Protector", cost(&[generic(7)]), 6, 8)
    }
}

/// Tomb of the Spirit Dragon — {T}: {C}; {2}, {T}: a life per colorless
/// creature.
pub fn tomb_of_the_spirit_dragon() -> CardDefinition {
    CardDefinition {
        name: "Tomb of the Spirit Dragon",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(
                        R::Creature.and(colorless_obj()).and(R::ControlledByYou),
                    ))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Twins of Discord — attacking shuts down blockers of one mana-value
/// parity; other colorless creatures of yours have bloodthirst 2. Residual:
/// the bloodthirst rides the cast.
pub fn twins_of_discord() -> CardDefinition {
    let cant_block = |odd: bool| Effect::MatchingCantBlockThisTurn { filter: R::Creature.and(R::ManaValueParity { odd }) };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::ChooseMode(vec![cant_block(true), cant_block(false)]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(colorless_obj()) },
                ),
                effect: Effect::If {
                    cond: Predicate::PlayerDamagedThisTurn { who: PlayerRef::EachOpponent },
                    then: Box::new(Effect::SpellEntersWithCounters {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..eldrazi("Twins of Discord", cost(&[generic(7)]), 8, 6)
    }
}

/// Ulalek, Fused Atrocity — an Eldrazi cast may pay {C}{C} to copy every
/// spell and ability you control.
pub fn ulalek_fused_atrocity() -> CardDefinition {
    devoid(CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Eldrazi),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {C}{C} to copy all spells and abilities you control?".into(),
                mana_cost: cost(&[colorless(1), colorless(1)]),
                body: Box::new(Effect::CopyAllSpellsAndAbilitiesYouControl),
                else_: None,
            },
        }],
        ..eldrazi(
            "Ulalek, Fused Atrocity",
            cost(&[
                colorless_hybrid(Color::White),
                colorless_hybrid(Color::Blue),
                colorless_hybrid(Color::Black),
                colorless_hybrid(Color::Red),
                colorless_hybrid(Color::Green),
            ]),
            2,
            5,
        )
    })
}

/// Ulamog's Dreadsire — vigilance; ward (sacrifice a nonzero-mana-value
/// permanent); {T}: a 10/10 Eldrazi.
pub fn ulamogs_dreadsire() -> CardDefinition {
    let titan = TokenDefinition {
        name: "Eldrazi".into(),
        power: 10,
        toughness: 10,
        card_types: vec![CardType::Creature],
        subtypes: kindred_eldrazi(),
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Ward(WardCost::SacrificeMatching(Box::new(R::Permanent.and(R::ManaValueAtLeast(1))))),
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(titan) },
            ..Default::default()
        }],
        ..eldrazi("Ulamog's Dreadsire", cost(&[generic(10)]), 10, 10)
    }
}

/// Wastescape Battlemage — kicker {G} exiles an opponent's artifact or
/// enchantment; kicker {1}{U} bounces an opponent's creature.
pub fn wastescape_battlemage() -> CardDefinition {
    let kicked = |i: u8, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource)
            .with_filter(Predicate::CastSpellWasKickedWith(i)),
        effect,
    };
    let mut def = CardDefinition {
        kicker_options: vec![cost(&[g()]), cost(&[generic(1), u()])],
        triggered_abilities: vec![
            kicked(
                0,
                Effect::Move {
                    what: target_filtered(R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent)),
                    to: ZoneDest::Exile,
                },
            ),
            kicked(
                1,
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
            ),
        ],
        ..eldrazi("Wastescape Battlemage", cost(&[generic(1), colorless(1)]), 2, 2)
    };
    def.subtypes.creature_types.push(CreatureType::Wizard);
    def
}
