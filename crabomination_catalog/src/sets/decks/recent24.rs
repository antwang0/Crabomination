//! A twenty-fourth wave — Aetherdrift (DFT) staples on existing primitives:
//! Vehicles + Crew, Mount/Vehicle anthems, cycling burn/removal, discard-count
//! triggers (`EventKind::CardDiscarded`), modal removal, distribute-counters,
//! and an exile-top "play it this turn" enchantment — plus a small Duskmourn
//! (DSK) tail of enchantment-creature staples. Tests in
//! `crabomination/src/tests/recent24.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    Effect, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    MayPlayDuration, Predicate, SelectionRequirement, Selector, StaticAbility, StaticEffect,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, draw, drain, etb, gain_life, pump_target, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

/// "target creature or Vehicle" target filter.
fn creature_or_vehicle() -> SelectionRequirement {
    SelectionRequirement::Creature
        .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle))
}

/// "creature or planeswalker" target filter.
fn creature_or_pw() -> SelectionRequirement {
    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker)
}

fn cycling(c: crate::mana::ManaCost) -> Keyword {
    Keyword::Cycling(c)
}

// ── Instants / sorceries ─────────────────────────────────────────────────────

/// Bounce Off — {U} Instant. Return target creature or Vehicle to its owner's hand.
pub fn bounce_off() -> CardDefinition {
    CardDefinition {
        name: "Bounce Off",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(creature_or_vehicle()),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        ..Default::default()
    }
}

/// Bestow Greatness — {2}{G} Instant. Target creature gets +4/+4 and gains trample.
pub fn bestow_greatness() -> CardDefinition {
    CardDefinition {
        name: "Bestow Greatness",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            pump_target(4, 4),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Broadside Barrage — {1}{U}{R} Instant. 5 damage to target creature or
/// planeswalker, then loot 1.
pub fn broadside_barrage() -> CardDefinition {
    CardDefinition {
        name: "Broadside Barrage",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            deal(5, target_filtered(creature_or_pw())),
            draw(1),
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]),
        ..Default::default()
    }
}

/// Crash and Burn — {3}{R} Instant. Destroy target Vehicle, or 6 damage to
/// target creature or planeswalker.
pub fn crash_and_burn() -> CardDefinition {
    CardDefinition {
        name: "Crash and Burn",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::HasArtifactSubtype(
                    ArtifactSubtype::Vehicle,
                )),
            },
            deal(6, target_filtered(creature_or_pw())),
        ]),
        ..Default::default()
    }
}

/// Spin Out — {1}{B}{B} Instant. Destroy target creature or Vehicle.
pub fn spin_out() -> CardDefinition {
    CardDefinition {
        name: "Spin Out",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy { what: target_filtered(creature_or_vehicle()) },
        ..Default::default()
    }
}

/// Syphon Fuel — {4}{B} Instant. Target creature gets -6/-6; you gain 2 life.
pub fn syphon_fuel() -> CardDefinition {
    CardDefinition {
        name: "Syphon Fuel",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-6),
                toughness: Value::Const(-6),
                duration: Duration::EndOfTurn,
            },
            gain_life(2),
        ]),
        ..Default::default()
    }
}

/// Locust Spray — {B} Instant. Target creature gets -1/-1. Cycling {B}.
pub fn locust_spray() -> CardDefinition {
    CardDefinition {
        name: "Locust Spray",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![cycling(cost(&[b()]))],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Skycrash — {1}{R} Instant. Destroy target artifact. Cycling {R}.
pub fn skycrash() -> CardDefinition {
    CardDefinition {
        name: "Skycrash",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![cycling(cost(&[r()]))],
        effect: Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
        ..Default::default()
    }
}

/// Maximum Overdrive — {1}{B} Instant. +1/+1 counter on target creature; it
/// gains deathtouch and indestructible until end of turn.
pub fn maximum_overdrive() -> CardDefinition {
    CardDefinition {
        name: "Maximum Overdrive",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Pedal to the Metal — {X}{R} Instant. Target creature gets +X/+0 and gains
/// first strike until end of turn.
pub fn pedal_to_the_metal() -> CardDefinition {
    CardDefinition {
        name: "Pedal to the Metal",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::XFromCost,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Fuel the Flames — {2}{R} Instant. 2 damage to each creature. Cycling {2}.
pub fn fuel_the_flames() -> CardDefinition {
    CardDefinition {
        name: "Fuel the Flames",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![cycling(cost(&[generic(2)]))],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(deal(2, Selector::TriggerSource)),
        },
        ..Default::default()
    }
}

/// Gallant Strike — {1}{W} Instant. Destroy target creature with toughness 4+.
/// Cycling {2}.
pub fn gallant_strike() -> CardDefinition {
    CardDefinition {
        name: "Gallant Strike",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![cycling(cost(&[generic(2)]))],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ToughnessAtLeast(4)),
            ),
        },
        ..Default::default()
    }
}

/// Risky Shortcut — {2}{B} Sorcery. Draw two cards. Each player loses 2 life.
pub fn risky_shortcut() -> CardDefinition {
    CardDefinition {
        name: "Risky Shortcut",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            draw(2),
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Road Rage — {R} Instant. X damage to target creature or planeswalker, where
/// X is 2 plus the number of Mounts and Vehicles you control.
pub fn road_rage() -> CardDefinition {
    let mounts_and_vehicles = Selector::EachPermanent(
        SelectionRequirement::ControlledByYou.and(
            SelectionRequirement::HasCreatureType(CreatureType::Mount)
                .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
        ),
    );
    CardDefinition {
        name: "Road Rage",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(creature_or_pw()),
            amount: Value::Sum(vec![Value::Const(2), Value::CountOf(Box::new(mounts_and_vehicles))]),
        },
        ..Default::default()
    }
}

/// Spectacular Pileup — {3}{W}{W} Sorcery. All creatures and Vehicles lose
/// indestructible, then destroy all creatures and Vehicles. Cycling {2}.
pub fn spectacular_pileup() -> CardDefinition {
    CardDefinition {
        name: "Spectacular Pileup",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![cycling(cost(&[generic(2)]))],
        effect: Effect::Seq(vec![
            Effect::LoseKeywordThisTurn {
                what: Selector::EachPermanent(creature_or_vehicle()),
                keyword: Keyword::Indestructible,
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(creature_or_vehicle()),
                body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
            },
        ]),
        ..Default::default()
    }
}

// ── Enchantments / auras ─────────────────────────────────────────────────────

/// Count on Luck — {R}{R}{R} Enchantment. At your upkeep, exile the top card;
/// you may play it this turn.
pub fn count_on_luck() -> CardDefinition {
    CardDefinition {
        name: "Count on Luck",
        cost: cost(&[r(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(1),
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

/// Silken Strength — {1}{G} Aura. Flash. Enchant creature or Vehicle. ETB untap
/// it. Enchanted permanent gets +1/+2 and has reach.
pub fn silken_strength() -> CardDefinition {
    CardDefinition {
        name: "Silken Strength",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(creature_or_vehicle()) },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 2,
            keywords: vec![Keyword::Reach],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Untap {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            up_to: None,
        })],
        ..Default::default()
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Nimble Thopterist — {3}{U} 3/2 Vedalken Artificer. ETB: make a 1/1 flying
/// Thopter.
pub fn nimble_thopterist() -> CardDefinition {
    CardDefinition {
        name: "Nimble Thopterist",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: thopter_token(),
        })],
        ..Default::default()
    }
}

/// Migrating Ketradon — {4}{G}{G} 6/6 Dinosaur. Reach. ETB gain 4 life.
/// Cycling {2}.
pub fn migrating_ketradon() -> CardDefinition {
    CardDefinition {
        name: "Migrating Ketradon",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Reach, cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(gain_life(4))],
        ..Default::default()
    }
}

/// Shefet Archfiend — {5}{B}{B} 5/5 Demon. Flying. ETB all other creatures get
/// -2/-2. Cycling {2}.
pub fn shefet_archfiend() -> CardDefinition {
    CardDefinition {
        name: "Shefet Archfiend",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
            ),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Lotusguard Disciple — {2}{W} 2/2 Bird Cleric. Flying. ETB: target creature
/// or Vehicle gains lifelink and indestructible until end of turn.
pub fn lotusguard_disciple() -> CardDefinition {
    CardDefinition {
        name: "Lotusguard Disciple",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(creature_or_vehicle()),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Fang Guardian — {3}{G} 4/2 Ape Druid. Flash. ETB another target creature or
/// Vehicle you control gets +2/+2 until end of turn.
pub fn fang_guardian() -> CardDefinition {
    CardDefinition {
        name: "Fang Guardian",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                creature_or_vehicle()
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Regal Imperiosaur — {1}{G}{G} 5/4 Dinosaur. Other Dinosaurs you control get
/// +1/+1.
pub fn regal_imperiosaur() -> CardDefinition {
    CardDefinition {
        name: "Regal Imperiosaur",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Other Dinosaurs you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Dinosaur)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Guidelight Synergist — {3}{W} 0/4 Robot Artificer. Flying. Gets +1/+0 for
/// each artifact you control.
pub fn guidelight_synergist() -> CardDefinition {
    CardDefinition {
        name: "Guidelight Synergist",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Artificer],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Guidelight Synergist gets +1/+0 for each artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: SelectionRequirement::Artifact,
                per_power: 1,
                per_toughness: 0,
            },
        }],
        ..Default::default()
    }
}

/// Cloudspire Captain — {2}{W} 2/3 Human Pilot. Mounts and Vehicles you control
/// get +1/+1; it saddles/crews as though its power were 2 greater.
pub fn cloudspire_captain() -> CardDefinition {
    CardDefinition {
        name: "Cloudspire Captain",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "Mounts and Vehicles you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::ControlledByYou.and(
                            SelectionRequirement::HasCreatureType(CreatureType::Mount)
                                .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                        ),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Saddles Mounts and crews Vehicles as though its power were 2 greater.",
                effect: StaticEffect::CrewSaddlePowerBonus { applies_to: Selector::This, amount: 2 },
            },
        ],
        ..Default::default()
    }
}

/// Daring Mechanic — {2}{W} 3/3 Human Artificer. {3}{W}: put a +1/+1 counter on
/// target Mount or Vehicle.
pub fn daring_mechanic() -> CardDefinition {
    CardDefinition {
        name: "Daring Mechanic",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Mount)
                        .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Deathless Pilot — {1}{B} 2/2 Zombie Pilot. Saddles/crews as though its power
/// were 2 greater. {3}{B}: return this card from your graveyard to your hand.
pub fn deathless_pilot() -> CardDefinition {
    CardDefinition {
        name: "Deathless Pilot",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Saddles Mounts and crews Vehicles as though its power were 2 greater.",
            effect: StaticEffect::CrewSaddlePowerBonus { applies_to: Selector::This, amount: 2 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gastal Blockbuster — {2}{R} 3/2 Human Berserker. ETB you may sacrifice a
/// creature or Vehicle; if you do, destroy target artifact an opponent controls.
pub fn gastal_blockbuster() -> CardDefinition {
    CardDefinition {
        name: "Gastal Blockbuster",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice a creature or Vehicle? (destroy an opponent's artifact)".into(),
            filter: creature_or_vehicle(),
            count: Value::Const(1),
            then: Box::new(Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByOpponent),
                ),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Scrounging Skyray — {1}{U} 1/2 Fish Pirate. Flying. Whenever you discard one
/// or more cards, put that many +1/+1 counters on it. Cycling {2}.
pub fn scrounging_skyray() -> CardDefinition {
    CardDefinition {
        name: "Scrounging Skyray",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Pactdoll Terror — {3}{B} 3/4 Toy artifact creature. Whenever this or another
/// artifact you control enters, each opponent loses 1 life and you gain 1.
pub fn pactdoll_terror() -> CardDefinition {
    CardDefinition {
        name: "Pactdoll Terror",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Toy], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: drain(1),
        }],
        ..Default::default()
    }
}

// ── Vehicles ─────────────────────────────────────────────────────────────────

/// Air Response Unit — {2}{W} 3/3 Vehicle. Flying, vigilance. Crew 1.
pub fn air_response_unit() -> CardDefinition {
    CardDefinition {
        name: "Air Response Unit",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Crew(1)],
        ..Default::default()
    }
}

/// Debris Beetle — {2}{B}{G} 6/6 Vehicle. Trample. ETB each opponent loses 3
/// life and you gain 3. Crew 2.
pub fn debris_beetle() -> CardDefinition {
    CardDefinition {
        name: "Debris Beetle",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample, Keyword::Crew(2)],
        triggered_abilities: vec![etb(drain(3))],
        ..Default::default()
    }
}

/// Cryptcaller Chariot — {3}{B} 5/5 Vehicle. Menace. Whenever you discard one
/// or more cards, create that many tapped 2/2 black Zombie tokens. Crew 2.
pub fn cryptcaller_chariot() -> CardDefinition {
    CardDefinition {
        name: "Cryptcaller Chariot",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Menace, Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: zombie_2_2_token(),
            },
        }],
        ..Default::default()
    }
}

/// Cloudspire Skycycle — {2}{R}{W} 2/3 Vehicle. Flying. ETB distribute two
/// +1/+1 counters among one or two other target Vehicles/creatures you control.
/// Crew 1.
pub fn cloudspire_skycycle() -> CardDefinition {
    CardDefinition {
        name: "Cloudspire Skycycle",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::DistributeCounters {
            total: Value::Const(2),
            counter: CounterType::PlusOnePlusOne,
            filter: creature_or_vehicle()
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
            max_targets: 2,
        })],
        ..Default::default()
    }
}

/// Thunderhead Gunner — {4}{R} 4/5 Shark Pirate. Reach. Discard a card: draw a
/// card (sorcery speed, once each turn).
pub fn thunderhead_gunner() -> CardDefinition {
    CardDefinition {
        name: "Thunderhead Gunner",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shark, CreatureType::Pirate],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((SelectionRequirement::Any, 1)),
            sorcery_speed: true,
            once_per_turn: true,
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wretched Doll — {1}{B} 3/1 Toy artifact creature. {B}, {T}: surveil 1.
pub fn wretched_doll() -> CardDefinition {
    CardDefinition {
        name: "Wretched Doll",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Toy], ..Default::default() },
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Molt Tender — {G} 1/1 Insect Druid. {T}: mill a card. {T}, exile a card from
/// your graveyard: add one mana of any color.
pub fn molt_tender() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Molt Tender",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Mill { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                exile_other_filter: Some((SelectionRequirement::Any, 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Scrap Compactor — {1} Artifact. {3}, {T}, Sacrifice this: 3 damage to a
/// creature. {6}, {T}, Sacrifice this: destroy a creature or Vehicle.
pub fn scrap_compactor() -> CardDefinition {
    CardDefinition {
        name: "Scrap Compactor",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_cost: true,
                effect: deal(3, target_filtered(SelectionRequirement::Creature)),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(6)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Destroy { what: target_filtered(creature_or_vehicle()) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Defend the Rider — {G} Instant. Choose one — your permanent gains hexproof
/// and indestructible; or create a 1/1 colorless Pilot.
pub fn defend_the_rider() -> CardDefinition {
    CardDefinition {
        name: "Defend the Rider",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::ControlledByYou),
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: pilot_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Full Throttle — {4}{R}{R} Sorcery. After this main phase, two additional
/// combat phases; at the beginning of each combat this turn, untap all
/// creatures that attacked this turn.
pub fn full_throttle() -> CardDefinition {
    CardDefinition {
        name: "Full Throttle",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AdditionalCombatPhaseAfterMain { count: Value::Const(2) },
            Effect::AtEachCombatThisTurn {
                body: Box::new(Effect::Untap {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::AttackedThisTurn),
                    ),
                    up_to: None,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Canyon Vaulter — {1}{W} 3/1 Kor Pilot. Whenever it crews a Vehicle or
/// saddles a Mount during your main phase, that permanent gains flying.
pub fn canyon_vaulter() -> CardDefinition {
    CardDefinition {
        name: "Canyon Vaulter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Pilot],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CrewsOrSaddles, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Reckless Velocitaur — {3}{R} 3/3 Minotaur Pilot. Whenever it crews a Vehicle
/// or saddles a Mount during your main phase, that permanent gets +2/+0 and
/// gains trample.
pub fn reckless_velocitaur() -> CardDefinition {
    CardDefinition {
        name: "Reckless Velocitaur",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Pilot],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CrewsOrSaddles, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Duskmourn (DSK) tail ─────────────────────────────────────────────────────

/// Emerge from the Cocoon — {4}{W} Sorcery. Return a creature card from your
/// graveyard to the battlefield; gain 3 life.
pub fn emerge_from_the_cocoon() -> CardDefinition {
    CardDefinition {
        name: "Emerge from the Cocoon",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            gain_life(3),
        ]),
        ..Default::default()
    }
}

/// Enter the Enigma — {U} Sorcery. Target creature can't be blocked this turn;
/// draw a card.
pub fn enter_the_enigma() -> CardDefinition {
    CardDefinition {
        name: "Enter the Enigma",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Exorcise — {1}{W} Sorcery. Exile target artifact, enchantment, or creature
/// with power 4 or greater.
pub fn exorcise() -> CardDefinition {
    CardDefinition {
        name: "Exorcise",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .or(SelectionRequirement::Creature
                        .and(SelectionRequirement::PowerAtLeast(4))),
            ),
        },
        ..Default::default()
    }
}

/// Fear of Lost Teeth — {B} 1/1 Nightmare. When it dies, it deals 1 damage to
/// any target and you gain 1 life.
pub fn fear_of_lost_teeth() -> CardDefinition {
    CardDefinition {
        name: "Fear of Lost Teeth",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Seq(vec![
            deal(1, crate::effect::shortcut::target_any()),
            gain_life(1),
        ]))],
        ..Default::default()
    }
}

/// Friendly Teddy — {2} 2/2 Bear Toy artifact creature. When it dies, each
/// player draws a card.
pub fn friendly_teddy() -> CardDefinition {
    CardDefinition {
        name: "Friendly Teddy",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Toy],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Draw {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Give In to Violence — {1}{B} Instant. Target creature gets +2/+2 and gains
/// lifelink until end of turn.
pub fn give_in_to_violence() -> CardDefinition {
    CardDefinition {
        name: "Give In to Violence",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            pump_target(2, 2),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Grasping Longneck — {2}{G} 4/2 Horror. Reach. When it dies, you gain 2 life.
pub fn grasping_longneck() -> CardDefinition {
    CardDefinition {
        name: "Grasping Longneck",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(gain_life(2))],
        ..Default::default()
    }
}

/// Horrid Vigor — {1}{G} Instant. Target creature gains deathtouch and
/// indestructible until end of turn.
pub fn horrid_vigor() -> CardDefinition {
    CardDefinition {
        name: "Horrid Vigor",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Glimmerburst — {3}{U} Instant. Draw two cards; create a 1/1 white Glimmer
/// enchantment creature token.
pub fn glimmerburst() -> CardDefinition {
    CardDefinition {
        name: "Glimmerburst",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            draw(2),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: glimmer_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Friendly Ghost — {3}{W} 2/4 Spirit. Flying. When it enters, target creature
/// gets +2/+4 until end of turn.
pub fn friendly_ghost() -> CardDefinition {
    CardDefinition {
        name: "Friendly Ghost",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

// ── Tokens ───────────────────────────────────────────────────────────────────

fn glimmer_token() -> TokenDefinition {
    TokenDefinition {
        name: "Glimmer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Glimmer], ..Default::default() },
        ..Default::default()
    }
}

fn pilot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pilot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pilot], ..Default::default() },
        ..Default::default()
    }
}

fn thopter_token() -> TokenDefinition {
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

fn zombie_2_2_token() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        tapped: true,
        ..Default::default()
    }
}
