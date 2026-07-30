//! Fifth Dawn (5DN) gap batch 1 — the Beacons, the sunburst/charge artifacts
//! and the cast-matters commons. Tests in `recent_b/fdn5`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        ..Default::default()
    }
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
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

fn artifact_creature(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, mana, power, toughness, types, keywords)
    }
}

fn spell(name: &'static str, mana: ManaCost, sorcery: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery {
            CardType::Sorcery
        } else {
            CardType::Instant
        }],
        effect,
        ..Default::default()
    }
}

/// The Fifth Dawn "Beacon" cycle: a big sorcery that shuffles itself back.
fn beacon(name: &'static str, mana: ManaCost, body: Effect) -> CardDefinition {
    spell(
        name,
        mana,
        true,
        Effect::Seq(vec![body, Effect::ShuffleSelfIntoLibrary]),
    )
}

/// "Whenever a player casts a spell, [effect]."
fn on_any_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
        effect,
    }
}

// ── Beacons ──

/// Beacon of Creation — an Insect per Forest, then back into the deck.
pub fn beacon_of_creation() -> CardDefinition {
    beacon(
        "Beacon of Creation",
        cost(&[generic(3), g()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CountOf(Box::new(Selector::EachPermanent(
                R::HasLandType(crate::card::LandType::Forest).and(R::ControlledByYou),
            ))),
            definition: TokenDefinition {
                name: "Insect".into(),
                colors: vec![Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Insect],
                    ..Default::default()
                },
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        },
    )
}

/// Beacon of Tomorrows — an extra turn for anyone, then back into the deck.
pub fn beacon_of_tomorrows() -> CardDefinition {
    beacon(
        "Beacon of Tomorrows",
        cost(&[generic(6), u(), u()]),
        Effect::TakeExtraTurn {
            who: PlayerRef::Target(0),
            count: Value::ONE,
        },
    )
}

/// Beacon of Unrest — reanimate anything artifact or creature, then reshuffle.
pub fn beacon_of_unrest() -> CardDefinition {
    beacon(
        "Beacon of Unrest",
        cost(&[generic(3), b(), b()]),
        Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::InGraveyard.and(R::Artifact.or(R::Creature)),
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
    )
}

// ── Artifacts ──

/// Baton of Courage — flash + sunburst; spend the counters as pumps.
pub fn baton_of_courage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Sunburst],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Baton of Courage", cost(&[generic(3)]))
    }
}

/// Clock of Omens — two artifacts tap to untap a third.
pub fn clock_of_omens() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Artifact, 2)),
            effect: Effect::Untap {
                what: target_filtered(R::Artifact),
                up_to: None,
            },
            ..Default::default()
        }],
        ..artifact("Clock of Omens", cost(&[generic(4)]))
    }
}

/// Gemstone Array — banks generic mana as charge counters, pays it back in any
/// colour.
pub fn gemstone_array() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                remove_counter_cost: Some((CounterType::Charge, 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..artifact("Gemstone Array", cost(&[generic(4)]))
    }
}

/// Energy Chamber — a counter a turn, on an artifact creature or an artifact.
pub fn energy_chamber() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Artifact.and(R::Creature)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::AddCounter {
                    what: target_filtered(R::Artifact.and(R::Noncreature)),
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..artifact("Energy Chamber", cost(&[generic(2)]))
    }
}

// ── Artifact creatures ──

/// Anodet Lurker — a 3/3 body that pays 3 life on the way out.
pub fn anodet_lurker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(3),
        })],
        ..artifact_creature(
            "Anodet Lurker",
            cost(&[generic(5)]),
            3,
            3,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

/// Arachnoid — a colourless reach wall.
pub fn arachnoid() -> CardDefinition {
    artifact_creature(
        "Arachnoid",
        cost(&[generic(6)]),
        2,
        6,
        vec![CreatureType::Spider],
        vec![Keyword::Reach],
    )
}

/// Ferropede — unblockable, and strips a counter on connect.
pub fn ferropede() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Remove a counter from target permanent?".into(),
                body: Box::new(Effect::RemoveAnyCounter {
                    what: target_filtered(R::Permanent),
                }),
            },
        }],
        ..artifact_creature(
            "Ferropede",
            cost(&[generic(3)]),
            1,
            1,
            vec![CreatureType::Insect],
            vec![Keyword::Unblockable],
        )
    }
}

/// Composite Golem — a 4/4 that cashes itself in for one of each colour.
pub fn composite_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![
                    Color::White,
                    Color::Blue,
                    Color::Black,
                    Color::Red,
                    Color::Green,
                ]),
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Composite Golem",
            cost(&[generic(6)]),
            4,
            4,
            vec![CreatureType::Golem],
            vec![],
        )
    }
}

// ── Creatures ──

/// Advanced Hoverguard — a flier that can duck out of removal.
pub fn advanced_hoverguard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Shroud,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Advanced Hoverguard",
            cost(&[generic(3), u()]),
            2,
            2,
            vec![CreatureType::Drone],
            vec![Keyword::Flying],
        )
    }
}

/// Blind Creeper — a 3/3 that shrinks with every spell cast.
pub fn blind_creeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_any_cast(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Blind Creeper",
            cost(&[generic(1), b()]),
            3,
            3,
            vec![CreatureType::Zombie, CreatureType::Beast],
            vec![],
        )
    }
}

/// Ebon Drake — a cheap flier that bleeds you on every cast.
pub fn ebon_drake() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_any_cast(Effect::LoseLife {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Ebon Drake",
            cost(&[generic(2), b()]),
            3,
            3,
            vec![CreatureType::Drake],
            vec![Keyword::Flying],
        )
    }
}

/// Desecration Elemental — an 8/8 for four that eats your board.
pub fn desecration_elemental() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_any_cast(Effect::Sacrifice {
            who: Selector::You,
            count: Value::ONE,
            filter: R::Creature,
        })],
        ..creature(
            "Desecration Elemental",
            cost(&[generic(3), b()]),
            8,
            8,
            vec![CreatureType::Elemental],
            vec![Keyword::Fear],
        )
    }
}

/// Cosmic Larva — a 7/6 for three that eats two lands a turn.
pub fn cosmic_larva() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::SelfSource,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::SacrificeSourceUnlessSacrifice { filter: R::Land },
        }],
        ..creature(
            "Cosmic Larva",
            cost(&[generic(1), r(), r()]),
            7,
            6,
            vec![CreatureType::Beast],
            vec![Keyword::Trample],
        )
    }
}

/// Fleshgrafter — pitches artifacts for a pump.
pub fn fleshgrafter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Artifact, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Fleshgrafter",
            cost(&[generic(2), b()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Goblin Brawler — first strike, and no Equipment will stick to it.
pub fn goblin_brawler() -> CardDefinition {
    creature(
        "Goblin Brawler",
        cost(&[generic(2), r()]),
        2,
        2,
        vec![CreatureType::Goblin, CreatureType::Warrior],
        vec![Keyword::FirstStrike, Keyword::CantBeEquipped],
    )
}

// ── Spells ──

/// Abuna's Chant — life or a shield. Entwine {2}.
pub fn abunas_chant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[generic(2)]))],
        ..spell(
            "Abuna's Chant",
            cost(&[generic(3), w()]),
            false,
            Effect::ChooseMode(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(5),
                },
                Effect::PreventNextDamage {
                    target: target_filtered(R::Creature),
                    amount: Value::Const(5),
                },
            ]),
        )
    }
}

/// Armed Response — the Equipment count, straight at an attacker.
pub fn armed_response() -> CardDefinition {
    spell(
        "Armed Response",
        cost(&[generic(2), w()]),
        false,
        Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking)),
            amount: Value::CountOf(Box::new(Selector::EachPermanent(
                R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou),
            ))),
        },
    )
}

/// Feedback Bolt — the artifact count, straight at a player.
pub fn feedback_bolt() -> CardDefinition {
    spell(
        "Feedback Bolt",
        cost(&[generic(4), r()]),
        false,
        Effect::DealDamage {
            to: Selector::Player(PlayerRef::Target(0)),
            amount: Value::CountOf(Box::new(Selector::EachPermanent(
                R::Artifact.and(R::ControlledByYou),
            ))),
        },
    )
}

/// Channel the Suns — one of each colour.
pub fn channel_the_suns() -> CardDefinition {
    spell(
        "Channel the Suns",
        cost(&[generic(3), g()]),
        true,
        Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![
                Color::White,
                Color::Blue,
                Color::Black,
                Color::Red,
                Color::Green,
            ]),
        },
    )
}

/// Devour in Shadow — unconditional removal you pay for in life.
pub fn devour_in_shadow() -> CardDefinition {
    spell(
        "Devour in Shadow",
        cost(&[b(), b()]),
        false,
        Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(target_filtered(R::Creature))),
            },
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature),
            },
        ]),
    )
}

/// Early Frost — tap up to three lands.
pub fn early_frost() -> CardDefinition {
    spell(
        "Early Frost",
        cost(&[generic(1), u()]),
        false,
        Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Land,
            effect: Box::new(Effect::Tap {
                what: Selector::Target(0),
            }),
        },
    )
}

/// Ferocious Charge — a big pump plus a dig.
pub fn ferocious_charge() -> CardDefinition {
    spell(
        "Ferocious Charge",
        cost(&[generic(2), g()]),
        false,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Fill with Fright — a two-card strip plus a dig.
pub fn fill_with_fright() -> CardDefinition {
    spell(
        "Fill with Fright",
        cost(&[generic(3), b()]),
        true,
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Blinkmoth Infusion — affinity, then every artifact untaps.
pub fn blinkmoth_infusion() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Affinity for artifacts.",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching {
                filter: R::Artifact,
                per: 1,
            },
        }],
        ..spell(
            "Blinkmoth Infusion",
            cost(&[generic(12), u(), u()]),
            false,
            Effect::Untap {
                what: Selector::EachPermanent(R::Artifact),
                up_to: None,
            },
        )
    }
}

// ── Enchantments ──

/// Dawn's Reflection — the enchanted land pays two extra mana.
pub fn dawns_reflection() -> CardDefinition {
    CardDefinition {
        name: "Dawn's Reflection",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land),
        },
        static_abilities: vec![StaticAbility {
            description: "Whenever enchanted land is tapped for mana, its controller adds an additional two mana in any combination of colors.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: true,
                filter: R::Land,
                extra: crate::effect::ExtraManaKind::AnyColors(2),
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}

/// Eyes of the Watcher — pay {1} on each instant or sorcery to dig two.
pub fn eyes_of_the_watcher() -> CardDefinition {
    CardDefinition {
        name: "Eyes of the Watcher",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_instant_or_sorcery()),
            effect: Effect::MayPay {
                description: "Pay {1} to scry 2?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
