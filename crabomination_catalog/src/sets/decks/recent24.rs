//! A twenty-fourth wave — Aetherdrift (DFT) staples on existing primitives:
//! Vehicles + Crew, Mount/Vehicle anthems, cycling burn/removal, discard-count
//! triggers (`EventKind::CardDiscarded`), modal removal, distribute-counters,
//! and an exile-top "play it this turn" enchantment — plus a small Duskmourn
//! (DSK) tail of enchantment-creature staples. Tests in
//! `crabomination/src/tests/recent24.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    Effect, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    MayPlayDuration, Predicate, SelectionRequirement, Selector, StaticAbility, StaticEffect,
    Subtypes, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{
    deal, draw, drain, each_opponent, eerie, etb, gain_life, pump_target,
    target_filtered,
};
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

/// Cult Healer — {2}{W} 3/3 Human Doctor. Eerie — gains lifelink until end
/// of turn whenever an enchantment you control enters or you fully unlock a
/// Room (DSK).
pub fn cult_healer() -> CardDefinition {
    CardDefinition {
        name: "Cult Healer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Doctor],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: eerie(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Lifelink,
            duration: Duration::EndOfTurn,
        }),
        ..Default::default()
    }
}

/// Balemurk Leech — {1}{B} 2/2 Leech. Eerie — each opponent loses 1 life
/// whenever an enchantment you control enters or you fully unlock a Room.
pub fn balemurk_leech() -> CardDefinition {
    CardDefinition {
        name: "Balemurk Leech",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leech], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: eerie(Effect::LoseLife {
            who: each_opponent(),
            amount: Value::Const(1),
        }),
        ..Default::default()
    }
}

/// Unwilling Vessel — {2}{U} 3/2 Human Wizard with vigilance. Eerie — put a
/// possession counter on it. When it dies, mint an X/X blue flying Spirit
/// where X is the number of counters on it (CR 603.10 LKI counter read).
pub fn unwilling_vessel() -> CardDefinition {
    let mut abilities = eerie(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Possession,
        amount: Value::Const(1),
    });
    abilities.push(crate::effect::shortcut::on_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: TokenDefinition {
            name: "Spirit".into(),
            card_types: vec![CardType::Creature],
            colors: vec![Color::Blue],
            subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
            keywords: vec![Keyword::Flying],
            dynamic_pt: Some((
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Possession,
                },
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Possession,
                },
            )),
            ..Default::default()
        },
    }));
    CardDefinition {
        name: "Unwilling Vessel",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: abilities,
        ..Default::default()
    }
}

/// Patched Plaything — {2}{W} 4/3 Toy artifact creature with double strike.
/// Enters with two -1/-1 counters if you cast it from your hand (CR 614.12 —
/// cast-zone-gated enters-with-counters).
pub fn patched_plaything() -> CardDefinition {
    CardDefinition {
        name: "Patched Plaything",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Toy], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::DoubleStrike],
        enters_with_counters: Some((
            CounterType::MinusOneMinusOne,
            Value::IfPred {
                pred: Box::new(Predicate::CastFromHand),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::ZERO),
            },
        )),
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

// ── Duskmourn (DSK) staples — wave 4 ─────────────────────────────────────────

/// A 1/1 red Gremlin creature token (DSK).
fn gremlin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Gremlin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gremlin], ..Default::default() },
        ..Default::default()
    }
}

/// Gremlin Tamer — {W}{U} 2/2 Human Scout. Eerie — create a 1/1 red Gremlin.
pub fn gremlin_tamer() -> CardDefinition {
    CardDefinition {
        name: "Gremlin Tamer",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: eerie(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: gremlin_token(),
        }),
        ..Default::default()
    }
}

/// Erratic Apparition — {2}{U} 1/3 Spirit with flying and vigilance. Eerie —
/// gets +1/+1 until end of turn.
pub fn erratic_apparition() -> CardDefinition {
    CardDefinition {
        name: "Erratic Apparition",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: eerie(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        }),
        ..Default::default()
    }
}

/// Diversion Specialist — {3}{R} 4/3 Human Warrior with menace. {1}, Sacrifice
/// another creature or enchantment: exile the top card of your library; you
/// may play it this turn.
pub fn diversion_specialist() -> CardDefinition {
    CardDefinition {
        name: "Diversion Specialist",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Enchantment)
                    .and(SelectionRequirement::OtherThanSource),
                1,
            )),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Clockwork Percussionist — {R} 1/1 Monkey Toy with haste. When it dies,
/// exile the top card of your library; you may play it until the end of your
/// next turn.
pub fn clockwork_percussionist() -> CardDefinition {
    CardDefinition {
        name: "Clockwork Percussionist",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Monkey, CreatureType::Toy],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                uncast_penalty: None,
            },
        )],
        ..Default::default()
    }
}

/// Commune with Evil — {2}{B} Sorcery. Look at the top four cards of your
/// library; put one into your hand and the rest into your graveyard, then gain
/// 3 life.
pub fn commune_with_evil() -> CardDefinition {
    CardDefinition {
        name: "Commune with Evil",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: true,
                pick_filter: None,
                take: None,
                to_battlefield: false,
            },
            gain_life(3),
        ]),
        ..Default::default()
    }
}

/// Sumala Sentry — {G}{W} 1/3 Elf Archer with reach. Whenever a face-down
/// permanent you control is turned face up, put a +1/+1 counter on it and on
/// Sumala Sentry.
pub fn sumala_sentry() -> CardDefinition {
    CardDefinition {
        name: "Sumala Sentry",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Archer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Cryptid Inspector — {2}{G} 2/3 Elf Warrior with vigilance. Whenever a
/// face-down permanent you control enters, and whenever a permanent you
/// control is turned face up, put a +1/+1 counter on Cryptid Inspector.
pub fn cryptid_inspector() -> CardDefinition {
    CardDefinition {
        name: "Cryptid Inspector",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::FaceDown,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Fanatic of the Harrowing — {3}{B} 2/2 Human Cleric. When it enters, each
/// player discards a card; if you discarded a card this way, draw a card.
pub fn fanatic_of_the_harrowing() -> CardDefinition {
    CardDefinition {
        name: "Fanatic of the Harrowing",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(1),
                random: false,
            },
            Effect::If {
                cond: Predicate::DiscardedThisEffect { who: PlayerRef::You },
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Spectral Snatcher — {4}{B}{B} 6/5 Spirit. Ward—Discard a card. Swampcycling {2}.
pub fn spectral_snatcher() -> CardDefinition {
    CardDefinition {
        name: "Spectral Snatcher",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 6,
        toughness: 5,
        keywords: vec![
            Keyword::Ward(WardCost::Discard(1)),
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Swamp),
        ],
        ..Default::default()
    }
}

/// Ghostly Keybearer — {3}{U} 3/3 Spirit with flying. Whenever it deals combat
/// damage to a player, unlock a locked door of up to one target Room you
/// control.
pub fn ghostly_keybearer() -> CardDefinition {
    CardDefinition {
        name: "Ghostly Keybearer",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::UnlockRoomDoor {
                what: target_filtered(
                    SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Room)
                        .and(SelectionRequirement::ControlledByYou),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Enduring Tenacity — {2}{B}{B} 4/3 Snake Glimmer enchantment creature.
/// Whenever you gain life, an opponent loses that much life. When it dies,
/// return it to the battlefield as an enchantment (Enduring).
pub fn enduring_tenacity() -> CardDefinition {
    CardDefinition {
        name: "Enduring Tenacity",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Glimmer],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::TriggerEventAmount,
                },
            },
            crate::effect::shortcut::on_dies(Effect::ReturnSelfAsEnchantment),
        ],
        ..Default::default()
    }
}

/// Threats Around Every Corner — {3}{G} Enchantment. ETB manifest dread.
/// Whenever a face-down permanent you control enters, search for a basic land
/// and put it onto the battlefield tapped.
pub fn threats_around_every_corner() -> CardDefinition {
    CardDefinition {
        name: "Threats Around Every Corner",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ManifestDread { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::FaceDown,
                    }),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            },
        ],
        ..Default::default()
    }
}

/// Insidious Fungus — {G} 1/2 Fungus. {2}, Sacrifice it: choose one — destroy
/// target artifact; destroy target enchantment; or draw a card. (The "may put
/// a land onto the battlefield" rider on the draw mode is dropped.)
pub fn insidious_fungus() -> CardDefinition {
    CardDefinition {
        name: "Insidious Fungus",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fungus], ..Default::default() },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::ChooseMode(vec![
                Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
                Effect::Destroy { what: target_filtered(SelectionRequirement::Enchantment) },
                draw(1),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Winter's Intervention — {1}{B} Instant. 2 damage to target creature; you
/// gain 2 life.
pub fn winters_intervention() -> CardDefinition {
    CardDefinition {
        name: "Winter's Intervention",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            deal(2, target_filtered(SelectionRequirement::Creature)),
            gain_life(2),
        ]),
        ..Default::default()
    }
}

/// Shroudstomper — {3}{W}{W}{B}{B} 5/5 Elemental with deathtouch. Whenever it
/// enters or attacks, each opponent loses 2 life; you gain 2 and draw a card.
pub fn shroudstomper() -> CardDefinition {
    let payoff = || Effect::Seq(vec![drain(2), draw(1)]);
    CardDefinition {
        name: "Shroudstomper",
        cost: cost(&[generic(3), w(), w(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            etb(payoff()),
            crate::effect::shortcut::on_attack(payoff()),
        ],
        ..Default::default()
    }
}

/// Sawblade Skinripper — {1}{B}{R} 3/2 Human Assassin with menace. {2},
/// Sacrifice another creature or enchantment: put a +1/+1 counter on it. At
/// your end step, if you sacrificed one or more permanents this turn, it deals
/// that much damage to any target.
pub fn sawblade_skinripper() -> CardDefinition {
    CardDefinition {
        name: "Sawblade Skinripper",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Enchantment)
                    .and(SelectionRequirement::OtherThanSource),
                1,
            )),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::PermanentsSacrificedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::ONE,
            }),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::PermanentsSacrificedThisTurn(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// A 4/4 white Beast token that can't attack or block alone (Toby's ETB).
fn lonely_beast_token() -> TokenDefinition {
    TokenDefinition {
        name: "Beast".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        keywords: vec![Keyword::CantAttackOrBlockAlone],
        ..Default::default()
    }
}

/// Toby, Beastie Befriender — {2}{W} 1/1 Legendary Human Wizard. ETB: make a
/// 4/4 white Beast that can't attack or block alone. As long as you control
/// four or more creature tokens, creature tokens you control have flying.
pub fn toby_beastie_befriender() -> CardDefinition {
    let creature_tokens_you_control = SelectionRequirement::Creature
        .and(SelectionRequirement::IsToken)
        .and(SelectionRequirement::ControlledByYou);
    CardDefinition {
        name: "Toby, Beastie Befriender",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: lonely_beast_token(),
        })],
        static_abilities: vec![StaticAbility {
            description: "While you control 4+ creature tokens, your creature tokens have flying.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(creature_tokens_you_control.clone()),
                    n: Value::Const(4),
                },
                applies_to: Selector::EachPermanent(creature_tokens_you_control),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..Default::default()
    }
}

/// A 2/2 green Spider creature token with reach (Twitching Doll).
fn spider_2_2_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spider".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Twitching Doll — {1}{G} 2/2 Artifact Creature — Spider Toy. {T}: add one
/// mana of any color and put a nest counter on it. {T}, Sacrifice it: make a
/// 2/2 green Spider with reach for each counter on it. Sorcery speed.
pub fn twitching_doll() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Twitching Doll",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Toy],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::ONE),
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Nest,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TotalCountersOn { what: Box::new(Selector::This) },
                    definition: spider_2_2_token(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Fear of Isolation — {1}{U} 2/3 Enchantment Creature — Nightmare with flying.
/// Additional cost: return a permanent you control to its owner's hand.
pub fn fear_of_isolation() -> CardDefinition {
    CardDefinition {
        name: "Fear of Isolation",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::ReturnToHand {
            filter: SelectionRequirement::ControlledByYou,
            count: 1,
        }],
        ..Default::default()
    }
}

/// Trapped in the Screen — {2}{W} Enchantment with ward {2}. When it enters,
/// exile target artifact, creature, or enchantment an opponent controls until
/// this leaves (CR 603.6e linked exile).
pub fn trapped_in_the_screen() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Trapped in the Screen",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::ControlledByOpponent.and(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .or(SelectionRequirement::Enchantment),
                ),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Sheltered by Ghosts — {1}{W} Aura. Enchant creature you control. ETB exile
/// target nonland permanent an opponent controls until this leaves. Enchanted
/// creature gets +1/+0 and has lifelink and ward {1}.
pub fn sheltered_by_ghosts() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Sheltered by Ghosts",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 0,
            keywords: vec![Keyword::Lifelink, Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByOpponent),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}
