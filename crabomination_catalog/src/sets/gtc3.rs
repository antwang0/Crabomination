//! Gatecrash (GTC) wave 3: the five guild Keyrunes, Extort creatures, combat
//! tricks, evasion beaters, and Auras on existing primitives. Tests in
//! `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, extort, target_any, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, Selector};
use crate::mana::{b, colored, cost, g, generic, hybrid, r, w, Color};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}
fn aura() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() }
}

// ── Guild Keyrunes ──────────────────────────────────────────────────────────

/// A guild Keyrune: {3} artifact tapping for one of two guild colors, and for
/// its guild-cost animating into the printed body until end of turn.
fn keyrune(
    name: &'static str,
    c1: Color,
    c2: Color,
    pt: (i32, i32),
    ct: CreatureType,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![c1, c2], Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[colored(c1), colored(c2)]),
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(pt.0),
                        toughness: Value::Const(pt.1),
                        creature_types: vec![ct],
                        keywords: kw,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::BecomeColor {
                        what: Selector::This,
                        colors: vec![c1, c2],
                        duration: Duration::EndOfTurn,
                        additive: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Boros Keyrune — animates into a 1/1 Soldier with double strike.
pub fn boros_keyrune() -> CardDefinition {
    keyrune("Boros Keyrune", Color::Red, Color::White, (1, 1), CreatureType::Soldier, vec![Keyword::DoubleStrike])
}
/// Dimir Keyrune — a 2/2 Horror that can't be blocked (modeled as EOT Unblockable).
pub fn dimir_keyrune() -> CardDefinition {
    keyrune("Dimir Keyrune", Color::Blue, Color::Black, (2, 2), CreatureType::Horror, vec![Keyword::Unblockable])
}
/// Gruul Keyrune — a 3/2 Beast with trample.
pub fn gruul_keyrune() -> CardDefinition {
    keyrune("Gruul Keyrune", Color::Red, Color::Green, (3, 2), CreatureType::Beast, vec![Keyword::Trample])
}
/// Orzhov Keyrune — a 1/4 Thrull with lifelink.
pub fn orzhov_keyrune() -> CardDefinition {
    keyrune("Orzhov Keyrune", Color::White, Color::Black, (1, 4), CreatureType::Thrull, vec![Keyword::Lifelink])
}
/// Simic Keyrune — a 2/3 Crab with hexproof.
pub fn simic_keyrune() -> CardDefinition {
    keyrune("Simic Keyrune", Color::Green, Color::Blue, (2, 3), CreatureType::Crab, vec![Keyword::Hexproof])
}

// ── Extort creatures ────────────────────────────────────────────────────────

/// Basilica Guards — {2}{W} 1/4 Human Soldier. Defender, Extort.
pub fn basilica_guards() -> CardDefinition {
    CardDefinition {
        name: "Basilica Guards",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![extort()],
        ..Default::default()
    }
}

/// Knight of Obligation — {3}{W} 2/4 Human Knight. Vigilance, Extort.
pub fn knight_of_obligation() -> CardDefinition {
    CardDefinition {
        name: "Knight of Obligation",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Knight]),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![extort()],
        ..Default::default()
    }
}

/// Syndicate Enforcer — {3}{B} 3/2 Human Rogue. Extort.
pub fn syndicate_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Syndicate Enforcer",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Rogue]),
        power: 3,
        toughness: 2,
        triggered_abilities: vec![extort()],
        ..Default::default()
    }
}

// ── Evasion beaters ─────────────────────────────────────────────────────────

/// Ripscale Predator — {4}{R}{R} 6/5 Dinosaur with menace.
pub fn ripscale_predator() -> CardDefinition {
    CardDefinition {
        name: "Ripscale Predator",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Dinosaur]),
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Menace],
        ..Default::default()
    }
}

/// Merfolk of the Depths — {4}{G/U}{G/U} 4/2 Merfolk Soldier with flash.
pub fn merfolk_of_the_depths() -> CardDefinition {
    CardDefinition {
        name: "Merfolk of the Depths",
        cost: cost(&[generic(4), hybrid(Color::Green, Color::Blue), hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Merfolk, CreatureType::Soldier]),
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        ..Default::default()
    }
}

/// Deathcult Rogue — {1}{U/B}{U/B} 2/2 Human Rogue; can't be blocked except by Rogues.
pub fn deathcult_rogue() -> CardDefinition {
    CardDefinition {
        name: "Deathcult Rogue",
        cost: cost(&[generic(1), hybrid(Color::Blue, Color::Black), hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Rogue]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(CreatureType::Rogue)))],
        ..Default::default()
    }
}

/// Spire Tracer — {G} 1/1 Elf Scout; can't be blocked except by flyers/reach.
pub fn spire_tracer() -> CardDefinition {
    CardDefinition {
        name: "Spire Tracer",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elf, CreatureType::Scout]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(
            R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Reach)),
        ))],
        ..Default::default()
    }
}

/// Spark Trooper — {1}{R}{R}{W} 6/1 Elemental Soldier with trample, lifelink,
/// haste; sacrificed at the beginning of the end step.
pub fn spark_trooper() -> CardDefinition {
    CardDefinition {
        name: "Spark Trooper",
        cost: cost(&[generic(1), r(), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elemental, CreatureType::Soldier]),
        power: 6,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Lifelink, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::SacrificeSource,
        }],
        ..Default::default()
    }
}

/// Urbis Protector — {4}{W}{W} 1/1 Human Cleric; ETB makes a 4/4 flying Angel.
pub fn urbis_protector() -> CardDefinition {
    CardDefinition {
        name: "Urbis Protector",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Cleric]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Angel".into(),
                power: 4,
                toughness: 4,
                keywords: vec![Keyword::Flying],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: creatures(vec![CreatureType::Angel]),
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Holy Mantle — {2}{W}{W} Aura. Enchanted creature gets +2/+2 and has
/// protection from creatures.
pub fn holy_mantle() -> CardDefinition {
    CardDefinition {
        name: "Holy Mantle",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::ProtectionFromCreatures],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Forced Adaptation — {G} Aura. At the beginning of your upkeep, put a +1/+1
/// counter on enchanted creature.
pub fn forced_adaptation() -> CardDefinition {
    CardDefinition {
        name: "Forced Adaptation",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::AddCounter {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

// ── Combat tricks & burn ────────────────────────────────────────────────────

/// Shielded Passage — {W} Instant. Prevent all damage that would be dealt to
/// target creature this turn.
pub fn shielded_passage() -> CardDefinition {
    CardDefinition {
        name: "Shielded Passage",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventAllDamageThisTurn { target: target_filtered(R::Creature) },
        ..Default::default()
    }
}

/// Furious Resistance — {R} Instant. Target blocking creature gets +3/+0 and
/// gains first strike until end of turn.
pub fn furious_resistance() -> CardDefinition {
    CardDefinition {
        name: "Furious Resistance",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsBlocking)),
                power: Value::Const(3),
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

/// Aerial Maneuver — {1}{W} Instant. Target creature gets +1/+1 and gains flying
/// and first strike until end of turn.
pub fn aerial_maneuver() -> CardDefinition {
    CardDefinition {
        name: "Aerial Maneuver",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Mugging — {R} Sorcery. Deals 2 damage to target creature. That creature
/// can't block this turn.
pub fn mugging() -> CardDefinition {
    CardDefinition {
        name: "Mugging",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Arrows of Justice — {2}{R/W} Instant. Deals 4 damage to target attacking or
/// blocking creature.
pub fn arrows_of_justice() -> CardDefinition {
    CardDefinition {
        name: "Arrows of Justice",
        cost: cost(&[generic(2), hybrid(Color::Red, Color::White)]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Homing Lightning — {2}{R}{R} Instant. Deals 4 damage to target creature and
/// each other creature with the same name.
pub fn homing_lightning() -> CardDefinition {
    CardDefinition {
        name: "Homing Lightning",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::SameNameDamage { subject: target_filtered(R::Creature), amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Massive Raid — {1}{R}{R} Instant. Deals damage to any target equal to the
/// number of creatures you control.
pub fn massive_raid() -> CardDefinition {
    CardDefinition {
        name: "Massive Raid",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_any(),
            amount: Value::CountOf(Box::new(Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature })),
        },
        ..Default::default()
    }
}

/// Ground Assault — {R}{G} Sorcery. Deals damage to target creature equal to the
/// number of lands you control.
pub fn ground_assault() -> CardDefinition {
    CardDefinition {
        name: "Ground Assault",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::CountOf(Box::new(Selector::ControlledBy { who: PlayerRef::You, filter: R::Land })),
        },
        ..Default::default()
    }
}
