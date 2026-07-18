//! MOM / BRO gap batch — vanilla-ish creatures, simple ETB/death/attack
//! triggers, firebreathing, and small removal/combat tricks, all on existing
//! primitives. Tests in `tests/recent_b/recent264.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// A self-source "when this dies" trigger.
fn dies(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect,
    }
}

// ── Tier 1: near-vanilla creatures + simple ETB/death triggers ───────────────

/// Alabaster Host Sanctifier — {1}{W} 2/2 Phyrexian Cleric with lifelink.
pub fn alabaster_host_sanctifier() -> CardDefinition {
    CardDefinition {
        name: "Alabaster Host Sanctifier",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    }
}

/// Nezumi Informant — {1}{B} 1/1 Rat Rogue. ETB: each opponent discards.
pub fn nezumi_informant() -> CardDefinition {
    CardDefinition {
        name: "Nezumi Informant",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
    }
}

/// Preening Champion — {2}{U} 2/2 Bird Knight with flying. ETB: make a 1/1
/// blue and red Elemental.
pub fn preening_champion() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Preening Champion",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: elemental,
        })],
        ..Default::default()
    }
}

/// Knight of the New Coalition — {3}{W} 2/2 Human Knight with vigilance. ETB:
/// make a 2/2 white and blue Knight with vigilance.
pub fn knight_of_the_new_coalition() -> CardDefinition {
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue],
        keywords: vec![Keyword::Vigilance],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Knight of the New Coalition",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: knight,
        })],
        ..Default::default()
    }
}

/// Conscripted Infantry — {2}{R} 3/1 Human Soldier. Dies: make a 1/1 colorless
/// Soldier artifact creature.
pub fn conscripted_infantry() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Conscripted Infantry",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: soldier,
        })],
        ..Default::default()
    }
}

/// Burrowing Razormaw — {2}{G} 4/2 Beast. Dies: mill four.
pub fn burrowing_razormaw() -> CardDefinition {
    CardDefinition {
        name: "Burrowing Razormaw",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![dies(Effect::Mill {
            who: Selector::You,
            amount: Value::Const(4),
        })],
        ..Default::default()
    }
}

/// Hoarding Recluse — {3}{G} 2/3 Spider with reach + deathtouch. Dies: put up
/// to one other target card from a graveyard on the bottom of its owner's
/// library.
pub fn hoarding_recluse() -> CardDefinition {
    CardDefinition {
        name: "Hoarding Recluse",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        triggered_abilities: vec![dies(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            }),
        })],
        ..Default::default()
    }
}

// ── Tier 2: firebreathing / simple activated abilities ───────────────────────

/// Fallaji Chaindancer — {3}{R} 2/4 Human Soldier. {2}: gains double strike EOT.
pub fn fallaji_chaindancer() -> CardDefinition {
    CardDefinition {
        name: "Fallaji Chaindancer",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Iridescent Blademaster — {1}{G} 2/2 Elf Warrior. {3}{G}: +2/+2 EOT.
pub fn iridescent_blademaster() -> CardDefinition {
    CardDefinition {
        name: "Iridescent Blademaster",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Air Marshal — {1}{U} 2/1 Human Soldier. {3}: target Soldier gains flying EOT.
pub fn air_marshal() -> CardDefinition {
    CardDefinition {
        name: "Air Marshal",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::HasCreatureType(CreatureType::Soldier)),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Onakke Javelineer — {4}{R} 5/4 Ogre Spirit with reach. {T}: 2 damage to
/// target player or battle.
pub fn onakke_javelineer() -> CardDefinition {
    CardDefinition {
        name: "Onakke Javelineer",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Spirit],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::HasCardType(CardType::Battle))),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dreg Recycler — {1}{B} 2/2 Phyrexian Beast. {T}, Sacrifice an artifact or
/// creature: each opponent loses 1 life and you gain 1.
pub fn dreg_recycler() -> CardDefinition {
    CardDefinition {
        name: "Dreg Recycler",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Tier 3: instants / sorceries ─────────────────────────────────────────────

/// Coming In Hot — {R} Instant. Target creature gets +1/+0 and first strike
/// EOT; scry 1.
pub fn coming_in_hot() -> CardDefinition {
    CardDefinition {
        name: "Coming In Hot",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Arachnoid Adaptation — {G} Instant. Target creature gets +2/+2 and reach
/// EOT, then untap it.
pub fn arachnoid_adaptation() -> CardDefinition {
    CardDefinition {
        name: "Arachnoid Adaptation",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Cosmic Hunger — {1}{G} Instant. Target creature you control deals damage
/// equal to its power to another target creature, planeswalker, or battle.
pub fn cosmic_hunger() -> CardDefinition {
    CardDefinition {
        name: "Cosmic Hunger",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamageEqualToPower {
            source: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::ControlledByYou),
            },
            target: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.or(R::Planeswalker).or(R::HasCardType(CardType::Battle)),
            },
        },
        ..Default::default()
    }
}

/// Mirrodin Avenged — {B} Instant. Destroy target creature that was dealt
/// damage this turn; draw a card.
pub fn mirrodin_avenged() -> CardDefinition {
    CardDefinition {
        name: "Mirrodin Avenged",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::DealtDamageThisTurn)),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Atraxa's Fall — {1}{G} Sorcery. Destroy target artifact, battle,
/// enchantment, or creature with flying.
pub fn atraxas_fall() -> CardDefinition {
    CardDefinition {
        name: "Atraxa's Fall",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(
                R::Artifact
                    .or(R::HasCardType(CardType::Battle))
                    .or(R::Enchantment)
                    .or(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            ),
        },
        ..Default::default()
    }
}

// ── Tier 4: moderate ─────────────────────────────────────────────────────────

/// Furnace Host Charger — {5}{R} 5/5 Phyrexian Giant with haste and
/// mountaincycling {2}.
pub fn furnace_host_charger() -> CardDefinition {
    CardDefinition {
        name: "Furnace Host Charger",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Giant],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![
            Keyword::Haste,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Mountain),
        ],
        ..Default::default()
    }
}

/// Phyrexian Pegasus — {2}{W} 2/2 Phyrexian Pegasus with flying. Attacks:
/// another target attacking creature without flying gains flying EOT.
pub fn phyrexian_pegasus() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Pegasus",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Pegasus],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(R::IsAttacking.and(R::HasKeyword(Keyword::Flying).negate())),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}
