//! Darksteel completion batch 2 — the rares (Memnarch, Death Cloud, Lich's
//! Tomb), the remaining artifact-matters uncommons, and the Pulse cycle.
//! Tests in `recent_b/dst`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Selector, SelectionRequirement as R,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, PlayerRef, Predicate, StaticAbility, StaticEffect, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, ManaCost};

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn creature(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, sorcery: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery { CardType::Sorcery } else { CardType::Instant }],
        effect,
        ..Default::default()
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        ..artifact(name, mana)
    }
}

// ── Creatures ──

/// Chromescale Drake — Affinity for artifacts, flying. On entry, reveal the
/// top three cards: artifacts to hand, the rest to the graveyard.
pub fn chromescale_drake() -> CardDefinition {
    CardDefinition {
        affinity_filter: Some(R::Artifact.and(R::ControlledByYou)),
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: Some(R::Artifact),
            take: Some(Value::Const(3)),
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            rest_bottom_random: false,
            picked_lands_to_battlefield: false,
        })],
        ..creature(
            "Chromescale Drake",
            cost(&[generic(6), u(), u(), u()]),
            3,
            4,
            vec![CreatureType::Drake],
            vec![Keyword::Flying],
        )
    }
}

/// Furnace Dragon — Affinity for artifacts, flying. On entry, exile all
/// artifacts. (The "if you cast it from your hand" rider is a cast check.)
pub fn furnace_dragon() -> CardDefinition {
    CardDefinition {
        affinity_filter: Some(R::Artifact.and(R::ControlledByYou)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::Exile { what: Selector::EachPermanent(R::Artifact) },
        }],
        ..creature(
            "Furnace Dragon",
            cost(&[generic(6), r(), r(), r()]),
            5,
            5,
            vec![CreatureType::Dragon],
            vec![Keyword::Flying],
        )
    }
}

/// Greater Harvester — your upkeep costs a permanent; its combat damage costs
/// the defender two.
pub fn greater_harvester() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Permanent,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Target(0)),
                    count: Value::Const(2),
                    filter: R::Permanent,
                },
            },
        ],
        ..creature(
            "Greater Harvester",
            cost(&[generic(2), b(), b(), b()]),
            5,
            6,
            vec![CreatureType::Horror],
            vec![],
        )
    }
}

/// Goblin Archaeologist — {R}, {T}: flip a coin. Heads destroys an artifact
/// and untaps it; tails eats it.
pub fn goblin_archaeologist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Seq(vec![
                    Effect::Destroy { what: target_filtered(R::Artifact) },
                    Effect::Untap { what: Selector::This, up_to: None },
                ])),
                on_tails: Box::new(Effect::SacrificeSource),
            },
            ..Default::default()
        }],
        ..creature(
            "Goblin Archaeologist",
            cost(&[generic(1), r()]),
            1,
            2,
            vec![CreatureType::Goblin, CreatureType::Artificer],
            vec![],
        )
    }
}

/// Drooling Ogre — whenever a player casts an artifact spell, that player
/// gains control of this creature.
pub fn drooling_ogre() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::Triggerer),
                duration: Duration::Permanent,
            },
        }],
        ..creature("Drooling Ogre", cost(&[generic(1), r()]), 3, 3, vec![CreatureType::Ogre], vec![])
    }
}

/// Neurok Transmuter — {U}: target creature becomes an artifact in addition
/// this turn. {U}: target artifact creature becomes blue and isn't an
/// artifact until end of turn.
pub fn neurok_transmuter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::AddCardTypeIndefinitely {
                    what: target_filtered(R::Creature),
                    card_type: CardType::Artifact,
                    until_eot: true,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::Seq(vec![
                    Effect::BecomeColor {
                        what: target_filtered(R::Artifact.and(R::Creature)),
                        colors: vec![Color::Blue],
                        additive: false,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::LoseCardTypeUntilEot {
                        what: Selector::Target(0),
                        card_type: CardType::Artifact,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature(
            "Neurok Transmuter",
            cost(&[generic(2), u()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Memnarch — {1}{U}{U}: target permanent becomes an artifact in addition to
/// its other types. {3}{U}: gain control of target artifact.
pub fn memnarch() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 4,
        toughness: 5,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u(), u()]),
                effect: Effect::AddCardTypeIndefinitely {
                    what: target_filtered(R::Permanent),
                    card_type: CardType::Artifact,
                    until_eot: false,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u()]),
                effect: Effect::GainControl {
                    what: target_filtered(R::Artifact),
                    to: None,
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
        ],
        ..artifact("Memnarch", cost(&[generic(7)]))
    }
}

// ── Artifacts ──

/// Chimeric Egg — charges off opponents' nonartifact casts; three charges
/// animate it into a 6/6 trampling Construct.
pub fn chimeric_egg() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                })),
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Charge, 3)),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(6),
                toughness: Value::Const(6),
                creature_types: vec![CreatureType::Construct],
                keywords: vec![Keyword::Trample],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Chimeric Egg", cost(&[generic(3)]))
    }
}

/// Geth's Grimoire — whenever an opponent discards a card, you may draw.
pub fn geths_grimoire() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..artifact("Geth's Grimoire", cost(&[generic(4)]))
    }
}

/// Talon of Pain — charges whenever a source you control damages an opponent;
/// {X}, {T}, remove X charge counters: X damage to any target. (The printed
/// "other than this artifact" exclusion isn't expressible on the scope, so its
/// own X shot re-charges it by one.)
pub fn talon_of_pain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourSourceDamagedOpponent),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            remove_counter_x: Some(CounterType::Charge),
            effect: Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
            ..Default::default()
        }],
        ..artifact("Talon of Pain", cost(&[generic(4)]))
    }
}

/// Thunderstaff — while untapped, shave 1 off each creature's combat damage
/// to you. {2}, {T}: attacking creatures get +1/+0 until end of turn.
pub fn thunderstaff() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "While untapped, prevent 1 combat damage each creature would deal you.",
            effect: StaticEffect::ReduceCombatDamageToControllerWhileUntapped(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::IsAttacking),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Thunderstaff", cost(&[generic(3)]))
    }
}

/// Wand of the Elements — sacrifice an Island for a 2/2 flying Elemental, or
/// a Mountain for a 3/3.
pub fn wand_of_the_elements() -> CardDefinition {
    let elemental = |power: i32, toughness: i32, color: Color, keywords: Vec<Keyword>| {
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Elemental".into(),
                colors: vec![color],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Elemental],
                    ..Default::default()
                },
                power,
                toughness,
                keywords,
                ..Default::default()
            },
        }
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((
                    R::HasLandType(crate::card::LandType::Island),
                    1,
                )),
                effect: elemental(2, 2, Color::Blue, vec![Keyword::Flying]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((
                    R::HasLandType(crate::card::LandType::Mountain),
                    1,
                )),
                effect: elemental(3, 3, Color::Red, vec![]),
                ..Default::default()
            },
        ],
        ..artifact("Wand of the Elements", cost(&[generic(4)]))
    }
}

/// Wirefly Hive — {3}, {T}: flip a coin. Heads mints a Wirefly; tails wipes
/// every permanent named Wirefly.
pub fn wirefly_hive() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Wirefly".into(),
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Insect],
                            ..Default::default()
                        },
                        power: 2,
                        toughness: 2,
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    },
                }),
                on_tails: Box::new(Effect::Destroy {
                    what: Selector::EachPermanent(R::HasName("Wirefly".into())),
                }),
            },
            ..Default::default()
        }],
        ..artifact("Wirefly Hive", cost(&[generic(3)]))
    }
}

/// Lich's Tomb — you don't lose the game at 0 life, but every point of life
/// lost costs you a permanent.
pub fn lichs_tomb() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You don't lose the game for having 0 or less life.",
            effect: StaticEffect::ControllerCantLoseGame,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::YourControl),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::TriggerEventAmount,
                filter: R::Permanent,
            },
        }],
        ..artifact("Lich's Tomb", cost(&[generic(4)]))
    }
}

/// Heartseeker — equipped creature gets +2/+1 and has "{T}, Unattach: Destroy
/// target creature."
pub fn heartseeker() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::IsHostOfSource),
            effect: Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature) },
                Effect::Unattach { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..equipment("Heartseeker", cost(&[generic(4)]), cost(&[generic(5)]))
    }
}

/// Auriok Siege Sled — {1}: force or forbid a block from an artifact creature.
/// (The forbid half is modeled as a blanket can't-block for the turn.)
pub fn auriok_siege_sled() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Juggernaut], ..Default::default() },
        power: 3,
        toughness: 5,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::MustBlockSource {
                    what: target_filtered(R::Artifact.and(R::Creature)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Artifact.and(R::Creature)),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..artifact("Auriok Siege Sled", cost(&[generic(6)]))
    }
}

/// Gemini Engine — attacking mints an attacking 3/4 Twin that is sacrificed at
/// end of combat. (The Twin's printed "P/T equal to this creature's" is fixed
/// at the Engine's base.)
pub fn gemini_engine() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                cleanup: crate::effect::AttackingTokenCleanup::SacrificeAtEndOfCombat,
                definition: TokenDefinition {
                    name: "Twin".into(),
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Construct],
                        ..Default::default()
                    },
                    power: 3,
                    toughness: 4,
                    ..Default::default()
                },
            },
        ]))],
        ..artifact("Gemini Engine", cost(&[generic(6)]))
    }
}

// ── Spells ──

/// Death Cloud — each player loses X life, discards X, then sacrifices X
/// creatures and X lands.
pub fn death_cloud() -> CardDefinition {
    CardDefinition {
        ..spell(
            "Death Cloud",
            cost(&[x(), b(), b(), b()]),
            true,
            Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::XFromCost,
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::XFromCost,
                    random: false,
                },
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    count: Value::XFromCost,
                    filter: R::Creature,
                },
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    count: Value::XFromCost,
                    filter: R::Land,
                },
            ]),
        )
    }
}

/// Murderous Spoils — destroy target nonblack creature; it can't be
/// regenerated and you take its Equipment.
pub fn murderous_spoils() -> CardDefinition {
    spell(
        "Murderous Spoils",
        cost(&[generic(5), b()]),
        false,
        Effect::Seq(vec![
            Effect::CantBeRegeneratedThisTurn {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
            Effect::GainControl {
                what: Selector::AttachedToMe(Box::new(Selector::Target(0))),
                to: None,
                duration: Duration::Permanent,
            },
            Effect::Destroy { what: Selector::Target(0) },
        ]),
    )
}

/// Test of Faith — prevent the next 3 damage to target creature; it grows a
/// +1/+1 counter per point prevented.
pub fn test_of_faith() -> CardDefinition {
    spell(
        "Test of Faith",
        cost(&[generic(1), w()]),
        false,
        Effect::PreventNextDamageWithCounters {
            target: target_filtered(R::Creature),
            amount: Value::Const(3),
        },
    )
}

/// Tears of Rage — attacking creatures get +X/+0, where X is the number of
/// attackers, then you sacrifice them at the next end step.
pub fn tears_of_rage() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::CurrentStepIs(
            crate::game::types::TurnStep::DeclareAttackers,
        )),
        ..spell(
            "Tears of Rage",
            cost(&[generic(2), r(), r()]),
            false,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::IsAttacking.and(R::ControlledByYou)),
                    power: Value::count(Selector::EachPermanent(R::IsAttacking)),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::SacrificeAllMatching {
                        who: Selector::You,
                        filter: R::Creature.and(R::AttackedThisTurn),
                    }),
                },
            ]),
        )
    }
}

/// "Then if [you're behind], return this to its owner's hand" — the Pulse
/// cycle's recursion rider.
fn pulse_rebuy(cond: Predicate) -> Effect {
    Effect::If {
        cond,
        then: Box::new(Effect::ReturnResolvingSpellToHand),
        else_: Box::new(Effect::Noop),
    }
}

/// Pulse of the Tangle — mint a 3/3 Beast, then bounce this if an opponent
/// controls more creatures.
pub fn pulse_of_the_tangle() -> CardDefinition {
    spell(
        "Pulse of the Tangle",
        cost(&[generic(1), g(), g()]),
        true,
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Beast".into(),
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Beast],
                        ..Default::default()
                    },
                    power: 3,
                    toughness: 3,
                    ..Default::default()
                },
            },
            pulse_rebuy(Predicate::AnOpponentControlsMoreCreatures),
        ]),
    )
}

/// Pulse of the Fields — gain 4 life, then bounce this if an opponent is
/// still ahead on life.
pub fn pulse_of_the_fields() -> CardDefinition {
    spell(
        "Pulse of the Fields",
        cost(&[generic(1), w(), w()]),
        false,
        Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            pulse_rebuy(Predicate::PlayerHasLessLifeThanOpponent { who: PlayerRef::You }),
        ]),
    )
}

/// Pulse of the Grid — draw two and discard one, then bounce this if an
/// opponent holds more cards than you.
pub fn pulse_of_the_grid() -> CardDefinition {
    spell(
        "Pulse of the Grid",
        cost(&[generic(1), u(), u()]),
        false,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            pulse_rebuy(Predicate::AnOpponentHasMoreCardsInHand),
        ]),
    )
}

/// Pulse of the Forge — 4 damage to a player or planeswalker, then bounce
/// this if they're still ahead on life.
pub fn pulse_of_the_forge() -> CardDefinition {
    spell(
        "Pulse of the Forge",
        cost(&[generic(1), r(), r()]),
        false,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::Const(4),
            },
            pulse_rebuy(Predicate::PlayerHasLessLifeThanOpponent { who: PlayerRef::You }),
        ]),
    )
}
