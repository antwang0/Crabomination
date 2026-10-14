//! Commander: one- and two-card gaps of EDHREC's commanders ranked 3501-4500
//! (the 2026-10-14 scan), fifth file. Tests in `tests/recent_b/cmdr_rank4k_e.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, Adventure, ArtifactSubtype, CardDefinition, CardType,
    CounterType, CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    MayPlayDuration, Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{blitz, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, w};
use std::sync::Arc;

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn plus_one(what: Selector) -> Effect {
    Effect::AddCounter {
        what,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    }
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT {
        what,
        power: Value::Const(p),
        toughness: Value::Const(t),
        duration: Duration::EndOfTurn,
    }
}

fn on_noncreature_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(crate::effect::shortcut::cast_is_noncreature()),
        effect,
    }
}

// ── G'raha Tia ───────────────────────────────────────────────────────────────

/// Murmuring Volume — {3} artifact — Book. {T}: one mana of any color. {2},
/// {T}, discard a card: draw a card.
pub fn murmuring_volume() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..spell("Murmuring Volume", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

// ── Meriadoc Brandybuck ──────────────────────────────────────────────────────

/// Pippin's Bravery — {G} instant. You may sacrifice a Food; if you do, target
/// creature gets +4/+4 until end of turn, otherwise +2/+2.
pub fn pippins_bravery() -> CardDefinition {
    spell(
        "Pippin's Bravery",
        cost(&[g()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump(target_filtered(R::Creature), 2, 2),
            Effect::MaySacrifice {
                description: "Sacrifice a Food for +4/+4 instead?".into(),
                filter: R::HasArtifactSubtype(ArtifactSubtype::Food),
                count: Value::ONE,
                then: Box::new(pump(Selector::Target(0), 2, 2)),
                else_: None,
            },
        ]),
    )
}

// ── Kari Zev, Skyship Raider ─────────────────────────────────────────────────

/// Pia, Determined Rebuilder — {2}{R} legendary 2/2 Human Artificer. ETB: a
/// 1/1 colorless flying Thopter artifact creature. {5}{R}: target creature
/// gets +X/+0 until end of turn, X = artifacts you control.
pub fn pia_determined_rebuilder() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(make(Value::ONE, thopter))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), r()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::PermanentCountControlledByMatching(PlayerRef::You, R::Artifact),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Pia, Determined Rebuilder",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// Collateral Damage — {R} instant. As an additional cost, sacrifice a
/// creature. 3 damage to any target.
pub fn collateral_damage() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..spell(
            "Collateral Damage",
            cost(&[r()]),
            CardType::Instant,
            Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: Value::Const(3),
            },
        )
    }
}

// ── Mai, Jaded Edge / Jessica Jones, Private Eye ─────────────────────────────

/// Dueling Rapier — {R} flash Equipment. ETB: attach it to target creature you
/// control. Equipped creature gets +2/+0. Equip {4}.
pub fn dueling_rapier() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(4)]))],
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            ..Default::default()
        }),
        ..spell("Dueling Rapier", cost(&[r()]), CardType::Artifact, Effect::Noop)
    }
}

/// Chandra's Emberling — {2}{R} 2/2 haste Gremlin Elemental. Whenever you cast
/// a noncreature spell, a +1/+1 counter on it.
pub fn chandras_emberling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_noncreature_cast(plus_one(Selector::This))],
        ..creature(
            "Chandra's Emberling",
            cost(&[generic(2), r()]),
            vec![CreatureType::Gremlin, CreatureType::Elemental],
            2,
            2,
        )
    }
}

/// Jessica Jones, Private Eye — {2}{R} legendary 2/3 Human Detective Hero.
/// {T}, put a stun counter on her: exile the top X cards of your library, X =
/// her power; you may play them this turn.
pub fn jessica_jones_private_eye() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            add_counter_cost: Some((CounterType::Stun, 1)),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::This)),
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..legend(
            "Jessica Jones, Private Eye",
            cost(&[generic(2), r()]),
            vec![
                CreatureType::Human,
                CreatureType::Detective,
                CreatureType::Hero,
            ],
            2,
            3,
        )
    }
}

// ── Mona Lisa, Ever Adaptable ────────────────────────────────────────────────

/// Mona Lisa, Ever Adaptable — {3}{G} legendary 4/4 Lizard Mutant. Whenever a
/// player casts a creature spell, you create a Mutagen token.
pub fn mona_lisa_ever_adaptable() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: make(Value::ONE, crabomination_base::tokens::mutagen_token()),
        }],
        ..legend(
            "Mona Lisa, Ever Adaptable",
            cost(&[generic(3), g()]),
            vec![CreatureType::Lizard, CreatureType::Mutant],
            4,
            4,
        )
    }
}

/// Cloakwood Swarmkeeper — {G} 1/1 Elf Ranger. Whenever one or more tokens you
/// control enter, a +1/+1 counter on it.
pub fn cloakwood_swarmkeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsToken,
                })
                .once_per_batch(),
            effect: plus_one(Selector::This),
        }],
        ..creature(
            "Cloakwood Swarmkeeper",
            cost(&[g()]),
            vec![CreatureType::Elf, CreatureType::Ranger],
            1,
            1,
        )
    }
}

// ── Yukora, the Prisoner ─────────────────────────────────────────────────────

/// Girder Goons — {4}{B} 4/4 Ogre Warrior. Dies: a tapped 2/2 black Rogue.
/// Blitz {3}{B}.
pub fn girder_goons() -> CardDefinition {
    let rogue = TokenDefinition {
        name: "Rogue".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rogue],
            ..Default::default()
        },
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: make(Value::ONE, rogue),
        }],
        alternative_cost: Some(blitz(cost(&[generic(3), b()]))),
        ..creature(
            "Girder Goons",
            cost(&[generic(4), b()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Zombie Ogre — {3}{B}{B} 3/5. At the beginning of your end step, if a
/// creature died this turn, venture into the dungeon.
pub fn zombie_ogre() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                    at_least: Value::ONE,
                }),
            effect: Effect::Venture,
        }],
        ..creature(
            "Zombie Ogre",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Ogre],
            3,
            5,
        )
    }
}

// ── Jugan, the Rising Star ───────────────────────────────────────────────────

/// Malachite Talisman — {2} artifact. Whenever a player casts a green spell,
/// you may pay {3}; if you do, untap target permanent.
pub fn malachite_talisman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Green))),
            effect: Effect::MayPay {
                description: "Pay {3} to untap a permanent?".into(),
                mana_cost: cost(&[generic(3)]),
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::Untap {
                        what: target_filtered(R::Permanent),
                        up_to: None,
                    }),
                }),
                else_: None,
            },
        }],
        ..spell("Malachite Talisman", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Gardenize — {1}{G}{G} enchantment. Whenever a creature you control dies, a
/// charge counter on it. At the beginning of your first main phase, add {G}
/// for each charge counter on it.
pub fn gardenize() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::PreCombatMain),
                    EventScope::YourControl,
                ),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(
                        Color::Green,
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        },
                    ),
                },
            },
        ],
        ..spell(
            "Gardenize",
            cost(&[generic(1), g(), g()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Appa, Loyal Sky Bison ────────────────────────────────────────────────────

/// Appa, Aang's Companion — {3}{W} legendary 2/4 flying Bison Ally. Whenever
/// it attacks, another target attacking creature without flying gains flying
/// until end of turn.
pub fn appa_aangs_companion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(
                R::Creature
                    .and(R::IsAttacking)
                    .and(R::OtherThanSource)
                    .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            ),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..legend(
            "Appa, Aang's Companion",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bison, CreatureType::Ally],
            2,
            4,
        )
    }
}

/// Momo, Rambunctious Rascal — {2}{W} legendary 1/1 flying Lemur Bat Ally.
/// ETB: 4 damage to target tapped creature an opponent controls.
pub fn momo_rambunctious_rascal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)),
            amount: Value::Const(4),
        })],
        ..legend(
            "Momo, Rambunctious Rascal",
            cost(&[generic(2), w()]),
            vec![CreatureType::Lemur, CreatureType::Bat, CreatureType::Ally],
            1,
            1,
        )
    }
}

// ── Rorix Bladewing ──────────────────────────────────────────────────────────

/// Brimstone Dragon — {6}{R}{R} 6/6 flying, haste Dragon.
pub fn brimstone_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        ..creature(
            "Brimstone Dragon",
            cost(&[generic(6), r(), r()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Smaug, the Great Calamity // Spew Flame — {5}{R}{R} legendary 5/5 flying
/// Dragon. Adventure ({4}{R} sorcery): 5 damage to target creature.
pub fn smaug_the_great_calamity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(Adventure {
            name: "Spew Flame",
            cost: cost(&[generic(4), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(5),
            },
        })),
        ..legend(
            "Smaug, the Great Calamity",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Dragon],
            5,
            5,
        )
    }
}
