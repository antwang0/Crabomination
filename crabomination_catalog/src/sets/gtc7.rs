//! Gatecrash (GTC) wave 7: the Simic Evolve package, Boros Battalion payoffs,
//! Gruul Bloodrush/land-scaling, and the Orzhov mode-sweep. Drives the new
//! Realmwright land-type static (`StaticEffect::LandsYouControlAreChosenType` +
//! `Effect::ChooseBasicLandTypeForSource`). Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, evolve, target_any, target_filtered};
use crate::effect::{
    Duration, ExtraManaKind, PlayerRef, Predicate, Selector, StaticEffect, ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}
fn aura() -> Subtypes {
    Subtypes {
        enchantment_subtypes: vec![EnchantmentSubtype::Aura],
        ..Default::default()
    }
}
fn creatures_you_control() -> Value {
    Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
        filter: R::Creature,
    }
}

// ── Simic (Evolve) ──────────────────────────────────────────────────────────

/// Crocanura — {2}{G} 1/3 Crocodile Frog. Reach, Evolve.
pub fn crocanura() -> CardDefinition {
    CardDefinition {
        name: "Crocanura",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Crocodile, CreatureType::Frog]),
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![evolve()],
        ..Default::default()
    }
}

/// Adaptive Snapjaw — {4}{G} 6/2 Lizard Beast. Evolve.
pub fn adaptive_snapjaw() -> CardDefinition {
    CardDefinition {
        name: "Adaptive Snapjaw",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Lizard, CreatureType::Beast]),
        power: 6,
        toughness: 2,
        triggered_abilities: vec![evolve()],
        ..Default::default()
    }
}

/// Battering Krasis — {2}{G} 2/1 Shark Beast. Trample, Evolve.
pub fn battering_krasis() -> CardDefinition {
    CardDefinition {
        name: "Battering Krasis",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Shark, CreatureType::Beast]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![evolve()],
        ..Default::default()
    }
}

/// Shambleshark — {G}{U} 2/1 Shark Crab. Flash, Evolve.
pub fn shambleshark() -> CardDefinition {
    CardDefinition {
        name: "Shambleshark",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Shark, CreatureType::Crab]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![evolve()],
        ..Default::default()
    }
}

/// Clinging Anemones — {3}{U} 1/4 Jellyfish. Defender, Evolve.
pub fn clinging_anemones() -> CardDefinition {
    CardDefinition {
        name: "Clinging Anemones",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Jellyfish]),
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![evolve()],
        ..Default::default()
    }
}

/// Simic Fluxmage — {2}{U} 1/2 Merfolk Wizard. Evolve. {1}{U}, {T}: Move a
/// +1/+1 counter from this creature onto target creature.
pub fn simic_fluxmage() -> CardDefinition {
    CardDefinition {
        name: "Simic Fluxmage",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Merfolk, CreatureType::Wizard]),
        power: 1,
        toughness: 2,
        triggered_abilities: vec![evolve()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::MoveCounter {
                from: Selector::This,
                to: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Renegade Krasis — {1}{G}{G} 3/2 Beast Mutant. Evolve; whenever this evolves,
/// put a +1/+1 counter on each other creature you control with a +1/+1 counter.
/// "Evolves" shares evolve's exact trigger condition, so the two abilities are
/// modeled as a paired trigger off the same greater-P/T ETB filter.
pub fn renegade_krasis() -> CardDefinition {
    let evolve_filter = R::Creature
        .and(R::OtherThanSource)
        .and(R::GreaterPowerOrToughnessThanSource);
    CardDefinition {
        name: "Renegade Krasis",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Beast, CreatureType::Mutant]),
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            evolve(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: evolve_filter,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::OtherThanSource)
                            .and(R::WithCounter(CounterType::PlusOnePlusOne)),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Species Gorger — {3}{G}{U} 6/6 Frog Beast. At the beginning of your upkeep,
/// return a creature you control to its owner's hand.
pub fn species_gorger() -> CardDefinition {
    CardDefinition {
        name: "Species Gorger",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Frog, CreatureType::Beast]),
        power: 6,
        toughness: 6,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        }],
        ..Default::default()
    }
}

/// Zameck Guildmage — {G}{U} 2/2 Elf Wizard. {G}{U}: this turn each creature
/// you control enters with an additional +1/+1 counter. {G}{U}, remove a
/// +1/+1 counter from a creature you control: Draw a card.
pub fn zameck_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Zameck Guildmage",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elf, CreatureType::Wizard]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g(), u()]),
                effect: Effect::GrantExtraPlusOneCountersThisTurn {
                    who: PlayerRef::You,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[g(), u()]),
                remove_counter_among_filter: Some((
                    Some(CounterType::PlusOnePlusOne),
                    1,
                    R::Creature.and(R::ControlledByYou),
                )),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Realmwright — {U} 1/1 Vedalken Wizard. As this enters, choose a basic land
/// type. Lands you control are that type in addition to their other types.
pub fn realmwright() -> CardDefinition {
    CardDefinition {
        name: "Realmwright",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Vedalken, CreatureType::Wizard]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ChooseBasicLandTypeForSource)],
        static_abilities: vec![StaticAbility {
            description: "Lands you control are the chosen type in addition to their other types.",
            effect: StaticEffect::LandsYouControlAreChosenType,
        }],
        ..Default::default()
    }
}

/// Miming Slime — {2}{G} Sorcery. Create an X/X green Ooze token, where X is
/// the greatest power among creatures you control.
pub fn miming_slime() -> CardDefinition {
    let x = Value::PowerOf(Box::new(Selector::GreatestPowerYouControl));
    CardDefinition {
        name: "Miming Slime",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Ooze".into(),
                power: 0,
                toughness: 0,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: creatures(vec![CreatureType::Ooze]),
                dynamic_pt: Some((x.clone(), x)),
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

// ── Gruul (Bloodrush / land-scaling) ─────────────────────────────────────────

/// Gruul Ragebeast — {5}{R}{G} 6/6 Beast. Whenever this or another creature you
/// control enters, that creature fights target creature an opponent controls.
pub fn gruul_ragebeast() -> CardDefinition {
    let fight = |attacker: Selector| Effect::Fight {
        attacker,
        defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
    };
    CardDefinition {
        name: "Gruul Ragebeast",
        cost: cost(&[generic(5), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Beast]),
        power: 6,
        toughness: 6,
        triggered_abilities: vec![
            // Self entering fights.
            etb(fight(Selector::This)),
            // Any other creature you control entering fights.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: fight(Selector::TriggerSource),
            },
        ],
        ..Default::default()
    }
}

/// Rubblebelt Maaka — {3}{R} 3/3 Cat. Bloodrush — {R}, Discard this: target
/// attacking creature gets +3/+3.
pub fn rubblebelt_maaka() -> CardDefinition {
    CardDefinition {
        name: "Rubblebelt Maaka",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Cat]),
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skarrg Guildmage — {R}{G} 2/2 Human Shaman. {R}{G}: creatures you control
/// gain trample EOT. {1}{R}{G}: target land you control becomes a 4/4
/// Elemental until end of turn (still a land).
pub fn skarrg_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Skarrg Guildmage",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Shaman]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r(), g()]),
                effect: Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r(), g()]),
                effect: Effect::BecomeCreature {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Human, CreatureType::Shaman],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hydroform — {G}{U} Instant. Target land becomes a 3/3 Elemental with flying
/// until end of turn (still a land).
pub fn hydroform() -> CardDefinition {
    CardDefinition {
        name: "Hydroform",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::BecomeCreature {
            what: target_filtered(R::Land),
            power: Value::Const(3),
            toughness: Value::Const(3),
            creature_types: vec![CreatureType::Elemental],
            keywords: vec![Keyword::Flying],
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Boros (Battalion / combat) ────────────────────────────────────────────────

/// Battalion trigger: fires when this and 2+ other creatures attack.
fn battalion(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
            .with_filter(Predicate::AttackingWithAtLeast(3)),
        effect,
    }
}

/// Foundry Champion — {4}{R}{W} 4/4 Elemental Soldier. ETB: deal damage to any
/// target equal to creatures you control. {R}: +1/+0 EOT. {W}: +0/+1 EOT.
pub fn foundry_champion() -> CardDefinition {
    CardDefinition {
        name: "Foundry Champion",
        cost: cost(&[generic(4), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elemental, CreatureType::Soldier]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_any(),
            amount: creatures_you_control(),
        })],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ZERO,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Viashino Firstblade — {1}{R}{W} 2/2 Lizard Soldier. Haste. ETB: it gets
/// +2/+2 until end of turn.
pub fn viashino_firstblade() -> CardDefinition {
    CardDefinition {
        name: "Viashino Firstblade",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Lizard, CreatureType::Soldier]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Ordruun Veteran — {2}{R}{W} 3/1 Minotaur Soldier. Battalion: gains double
/// strike EOT.
pub fn ordruun_veteran() -> CardDefinition {
    CardDefinition {
        name: "Ordruun Veteran",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Minotaur, CreatureType::Soldier]),
        power: 3,
        toughness: 1,
        triggered_abilities: vec![battalion(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::DoubleStrike,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Fortress Cyclops — {3}{R}{W} 3/3 Cyclops Soldier. Whenever it attacks it
/// gets +3/+0 EOT; whenever it blocks it gets +0/+3 EOT.
pub fn fortress_cyclops() -> CardDefinition {
    CardDefinition {
        name: "Fortress Cyclops",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Cyclops, CreatureType::Soldier]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ZERO,
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

// ── Green / Orzhov ────────────────────────────────────────────────────────────

/// Rust Scarab — {4}{G} 4/5 Insect. Whenever it becomes blocked, you may
/// destroy target artifact or enchantment an opponent controls.
pub fn rust_scarab() -> CardDefinition {
    CardDefinition {
        name: "Rust Scarab",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Insect]),
        power: 4,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Destroy target artifact or enchantment an opponent controls?".into(),
                body: Box::new(Effect::Destroy {
                    what: target_filtered(
                        R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent),
                    ),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Merciless Eviction — {4}{W}{B} Sorcery. Choose one — exile all artifacts /
/// creatures / enchantments / planeswalkers.
pub fn merciless_eviction() -> CardDefinition {
    let exile_all = |req: R| Effect::Exile {
        what: Selector::EachPermanent(req),
    };
    CardDefinition {
        name: "Merciless Eviction",
        cost: cost(&[generic(4), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            exile_all(R::Artifact),
            exile_all(R::Creature),
            exile_all(R::Enchantment),
            exile_all(R::Planeswalker),
        ]),
        ..Default::default()
    }
}

// ── Land Auras ────────────────────────────────────────────────────────────────

/// Verdant Haven — {2}{G} Aura. Enchant land. ETB: gain 2 life. Enchanted land
/// tapped for mana adds an additional one mana of any color.
pub fn verdant_haven() -> CardDefinition {
    CardDefinition {
        name: "Verdant Haven",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land),
        },
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land tapped for mana adds an additional one mana of any color.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: true,
                filter: R::Any,
                extra: ExtraManaKind::AnyColor,
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}

/// Skygames — {1}{U} Aura. Enchant land. Enchanted land has "{T}: Target
/// creature gains flying until end of turn. Activate only as a sorcery."
pub fn skygames() -> CardDefinition {
    CardDefinition {
        name: "Skygames",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land),
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Target creature gains flying EOT (sorcery speed).\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    sorcery_speed: true,
                    effect: Effect::GrantKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}
