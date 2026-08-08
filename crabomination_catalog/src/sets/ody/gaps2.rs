//! Odyssey (ODY) gap-closing wave 2: the Threshold Auras, the flashback burn
//! and the graveyard-cost shell. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{etb, target_any, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

/// The shared Aura shell: "Enchant creature", plus whatever the card grants.
fn aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

fn squirrel() -> TokenDefinition {
    TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Squirrel], ..Default::default() },
        ..Default::default()
    }
}

// ── The Desire Aura cycle ───────────────────────────────────────────────────

/// Aboshan's Desire — {U} flying, plus shroud past Threshold.
pub fn aboshans_desire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Threshold — enchanted creature has shroud.",
            effect: StaticEffect::PumpTeamIf {
                condition: threshold(),
                applies_to: Selector::attached_to(Selector::This),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Shroud],
            },
        }],
        ..aura(
            "Aboshan's Desire",
            cost(&[u()]),
            EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() },
        )
    }
}

/// Kamahl's Desire — {1}{R} first strike, plus +3/+0 past Threshold.
pub fn kamahls_desire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Threshold — enchanted creature gets +3/+0.",
            effect: StaticEffect::PumpTeamIf {
                condition: threshold(),
                applies_to: Selector::attached_to(Selector::This),
                power: 3,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..aura(
            "Kamahl's Desire",
            cost(&[generic(1), r()]),
            EquipBonus { keywords: vec![Keyword::FirstStrike], ..Default::default() },
        )
    }
}

/// Patriarch's Desire — {3}{B} +2/-2, doubled past Threshold.
pub fn patriarchs_desire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Threshold — enchanted creature gets an additional +2/-2.",
            effect: StaticEffect::PumpTeamIf {
                condition: threshold(),
                applies_to: Selector::attached_to(Selector::This),
                power: 2,
                toughness: -2,
                keywords: vec![],
            },
        }],
        ..aura(
            "Patriarch's Desire",
            cost(&[generic(3), b()]),
            EquipBonus { power: 2, toughness: -2, ..Default::default() },
        )
    }
}

/// Primal Frenzy — {G} trample.
pub fn primal_frenzy() -> CardDefinition {
    aura(
        "Primal Frenzy",
        cost(&[g()]),
        EquipBonus { keywords: vec![Keyword::Trample], ..Default::default() },
    )
}

/// Psionic Gift — {1}{U} turns the enchanted creature into a pinger.
pub fn psionic_gift() -> CardDefinition {
    aura(
        "Psionic Gift",
        cost(&[generic(1), u()]),
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Caustic Tar — {4}{B}{B} makes the enchanted land a drain engine.
pub fn caustic_tar() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(3),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..enchantment("Caustic Tar", cost(&[generic(4), b(), b()]))
    }
}

/// Squirrel Nest — {1}{G}{G} makes the enchanted land a Squirrel factory.
pub fn squirrel_nest() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(squirrel()),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..enchantment("Squirrel Nest", cost(&[generic(1), g(), g()]))
    }
}

/// Druid's Call — {1}{G} turns damage on the enchanted creature into Squirrels.
pub fn druids_call() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: Box::new(squirrel()),
                },
            }],
            ..Default::default()
        }),
        ..enchantment("Druid's Call", cost(&[generic(1), g()]))
    }
}

// ── Threshold creatures ─────────────────────────────────────────────────────

/// Metamorphic Wurm — {3}{G}{G} 3/3 that doubles past Threshold.
pub fn metamorphic_wurm() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Threshold — this creature gets +4/+4.",
            effect: StaticEffect::PumpSelfIf {
                condition: threshold(),
                power: 4,
                toughness: 4,
                keywords: vec![],
            },
        }],
        ..creature(
            "Metamorphic Wurm",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elephant, CreatureType::Wurm],
            3,
            3,
        )
    }
}

/// Gorilla Titan — {3}{G}{G} 4/4 trampler, 8/8 while your graveyard is empty.
pub fn gorilla_titan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +4/+4 as long as your graveyard is empty.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtMost(
                    Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Any },
                    Value::Const(0),
                ),
                power: 4,
                toughness: 4,
                keywords: vec![],
            },
        }],
        ..creature("Gorilla Titan", cost(&[generic(3), g(), g()]), vec![CreatureType::Ape], 4, 4)
    }
}

/// Crashing Centaur — {4}{G}{G} 3/4 that pitches for trample and hides past
/// Threshold.
pub fn crashing_centaur() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Threshold — this creature gets +2/+2 and has shroud.",
            effect: StaticEffect::PumpSelfIf {
                condition: threshold(),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Shroud],
            },
        }],
        ..creature(
            "Crashing Centaur",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Centaur],
            3,
            4,
        )
    }
}

/// Dirty Wererat — {3}{B} 2/3 that pitches to regenerate and swells past
/// Threshold.
pub fn dirty_wererat() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Threshold — this creature gets +2/+2 and can't block.",
            effect: StaticEffect::PumpSelfIf {
                condition: threshold(),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::CantBlock],
            },
        }],
        ..creature(
            "Dirty Wererat",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Rat, CreatureType::Minion],
            2,
            3,
        )
    }
}

/// Divine Sacrament — {1}{W}{W} a white anthem that doubles past Threshold.
pub fn divine_sacrament() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "White creatures get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::White))),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Threshold — white creatures get an additional +1/+1.",
                effect: StaticEffect::PumpTeamIf {
                    condition: threshold(),
                    applies_to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::White))),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                },
            },
        ],
        ..enchantment("Divine Sacrament", cost(&[generic(1), w(), w()]))
    }
}

/// Thermal Blast — {4}{R} 3 damage, 5 past Threshold.
pub fn thermal_blast() -> CardDefinition {
    instant(
        "Thermal Blast",
        cost(&[generic(4), r()]),
        Effect::If {
            cond: threshold(),
            then: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(5),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(3),
            }),
        },
    )
}

// ── Flashback burn and ramp ─────────────────────────────────────────────────

/// Volcanic Spray — {1}{R} a ground sweeper that comes back once.
pub fn volcanic_spray() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), r()]))],
        ..sorcery(
            "Volcanic Spray",
            cost(&[generic(1), r()]),
            Effect::DealDamage {
                to: Selector::Both(
                    Box::new(Selector::EachPermanent(
                        R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                    )),
                    Box::new(Selector::Player(PlayerRef::EachPlayer)),
                ),
                amount: Value::ONE,
            },
        )
    }
}

/// Earth Rift — {3}{R} Stone Rain with a heavy flashback.
pub fn earth_rift() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), r(), r()]))],
        ..sorcery(
            "Earth Rift",
            cost(&[generic(3), r()]),
            Effect::Destroy { what: target_filtered(R::Land) },
        )
    }
}

/// Scorching Missile — {3}{R} 4 to the face, with a nine-mana flashback.
pub fn scorching_missile() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(9), r()]))],
        ..sorcery(
            "Scorching Missile",
            cost(&[generic(3), r()]),
            Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::Const(4),
            },
        )
    }
}

/// Engulfing Flames — {R} a ping that shuts off regeneration.
pub fn engulfing_flames() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r()]))],
        ..instant(
            "Engulfing Flames",
            cost(&[r()]),
            Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
                Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Creature) },
            ]),
        )
    }
}

/// Elephant Ambush — {2}{G}{G} an instant-speed 3/3, twice.
pub fn elephant_ambush() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(6), g(), g()]))],
        ..instant(
            "Elephant Ambush",
            cost(&[generic(2), g(), g()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Elephant".into(),
                    power: 3,
                    toughness: 3,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Elephant],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        )
    }
}

/// Deep Reconnaissance — {2}{G} a tapped basic, twice.
pub fn deep_reconnaissance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), g()]))],
        ..sorcery(
            "Deep Reconnaissance",
            cost(&[generic(2), g()]),
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        )
    }
}

// ── The rest of the shell ───────────────────────────────────────────────────

/// Tremble — {1}{R} each player sacrifices a land.
pub fn tremble() -> CardDefinition {
    sorcery(
        "Tremble",
        cost(&[generic(1), r()]),
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: R::Land,
        },
    )
}

/// Pardic Miner — {1}{R} 1/1 that trades itself for a land drop.
pub fn pardic_miner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PlayerCantPlayLandsThisTurn { player: PlayerRef::Target(0) },
            ..Default::default()
        }],
        ..creature("Pardic Miner", cost(&[generic(1), r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

/// Price of Glory — {2}{R} punishes tapping lands on someone else's turn.
pub fn price_of_glory() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(
                    PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                )))),
            effect: Effect::Destroy { what: Selector::TriggerSource },
        }],
        ..enchantment("Price of Glory", cost(&[generic(2), r()]))
    }
}

/// Barbarian Lunatic — {2}{R} 2/1 that throws itself at a creature.
pub fn barbarian_lunatic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Barbarian Lunatic",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            2,
            1,
        )
    }
}

/// Minotaur Explorer — {1}{R} 3/3 that eats a random card or itself.
pub fn minotaur_explorer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CardsInHandMatching { who: PlayerRef::You, filter: R::Any },
                Value::ONE,
            ),
            then: Box::new(Effect::Discard {
                who: Selector::You,
                amount: Value::ONE,
                random: true,
            }),
            else_: Box::new(Effect::SacrificeSource),
        })],
        ..creature(
            "Minotaur Explorer",
            cost(&[generic(1), r()]),
            vec![CreatureType::Minotaur, CreatureType::Scout],
            3,
            3,
        )
    }
}

/// Overeager Apprentice — {2}{B} 1/2 that pitches a card for {B}{B}{B}.
pub fn overeager_apprentice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Black], Value::Const(3)),
            },
            ..Default::default()
        }],
        ..creature(
            "Overeager Apprentice",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            1,
            2,
        )
    }
}

/// Famished Ghoul — {3}{B} 3/2 that eats two cards out of one graveyard.
pub fn famished_ghoul() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::InGraveyard,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Exile,
                }),
            },
            ..Default::default()
        }],
        ..creature("Famished Ghoul", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 3, 2)
    }
}

/// Cabal Inquisitor — {1}{B} 1/1 whose Threshold discard eats your graveyard.
pub fn cabal_inquisitor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            sorcery_speed: true,
            condition: Some(threshold()),
            exile_other_filter: Some((R::InYourGraveyard, 2)),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Cabal Inquisitor",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            1,
            1,
        )
    }
}

/// Bearscape — {1}{G}{G} turns your graveyard into 2/2 Bears.
pub fn bearscape() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            exile_other_filter: Some((R::InYourGraveyard, 2)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Bear".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Bear],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..enchantment("Bearscape", cost(&[generic(1), g(), g()]))
    }
}

/// Ground Seal — {1}{G} cantrips and locks every graveyard out of targeting.
pub fn ground_seal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(crate::effect::shortcut::draw(1))],
        static_abilities: vec![StaticAbility {
            description: "Cards in graveyards can't be the targets of spells or abilities.",
            effect: StaticEffect::GraveyardCardsUntargetable,
        }],
        ..enchantment("Ground Seal", cost(&[generic(1), g()]))
    }
}

/// Nantuko Mentor — {2}{G} 1/1 that doubles a creature's power.
pub fn nantuko_mentor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::PowerOf(Box::new(Selector::Target(0))),
                toughness: Value::PowerOf(Box::new(Selector::Target(0))),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Nantuko Mentor",
            cost(&[generic(2), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Twigwalker — {2}{G} 2/2 that trades itself for a double pump.
pub fn twigwalker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
            },
            ..Default::default()
        }],
        ..creature("Twigwalker", cost(&[generic(2), g()]), vec![CreatureType::Insect], 2, 2)
    }
}

/// Skyshooter — {1}{G} 1/2 reach that snipes a combat flier.
pub fn skyshooter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    R::Creature
                        .and(R::HasKeyword(Keyword::Flying))
                        .and(R::IsAttacking.or(R::IsBlocking)),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Skyshooter",
            cost(&[generic(1), g()]),
            vec![CreatureType::Centaur, CreatureType::Archer],
            1,
            2,
        )
    }
}

/// Battle of Wits — {3}{U}{U} wins the game off a 200-card library.
pub fn battle_of_wits() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::ValueAtLeast(
                Value::LibrarySizeOf(PlayerRef::You),
                Value::Const(200),
            )),
            effect: Effect::WinGame { who: PlayerRef::You },
        }],
        ..enchantment("Battle of Wits", cost(&[generic(3), u(), u()]))
    }
}

/// Extract — {U} pulls one card out of a library for good.
pub fn extract() -> CardDefinition {
    sorcery(
        "Extract",
        cost(&[u()]),
        Effect::Search { who: PlayerRef::Target(0), filter: R::Any, to: ZoneDest::Exile },
    )
}
