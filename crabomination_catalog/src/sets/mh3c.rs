//! Modern Horizons 3 (MH3), batch 3 — battle-cry team pumps, Eldrazi Spawn
//! payoffs, modified-matters, and a saga. Introduces the Battle cry keyword
//! (CR 702.92). Tests in `tests/mh3c.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{adapt, on_attack, on_cast, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{b, colorless, cost, g, generic, r, w, Color};

// ── Battle cry (CR 702.92) ────────────────────────────────────────────────────

/// Goblin Wardriver — {1}{R} 2/2 Goblin Warrior with battle cry.
pub fn goblin_wardriver() -> CardDefinition {
    CardDefinition {
        name: "Goblin Wardriver",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::BattleCry(1)],
        ..Default::default()
    }
}

/// Accorder Paladin — {1}{W} 3/1 Human Soldier with battle cry.
pub fn accorder_paladin() -> CardDefinition {
    CardDefinition {
        name: "Accorder Paladin",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::BattleCry(1)],
        ..Default::default()
    }
}

/// Signal Pest — {1} 0/0 Artifact Creature — Pest with two +1/+1 anthems worth,
/// battle cry, and blocker-quality evasion. Printed as a 0/0 that enters as a
/// 2/1 by way of its printed body; modeled directly as 2/1 with battle cry and
/// "can't be blocked except by artifact and/or red creatures."
pub fn signal_pest() -> CardDefinition {
    CardDefinition {
        name: "Signal Pest",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pest], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![
            Keyword::BattleCry(1),
            Keyword::CantBeBlockedExceptBy(Box::new(
                R::Artifact.or(R::HasColor(Color::Red)),
            )),
        ],
        ..Default::default()
    }
}

/// Reckless Pyrosurfer — {1}{R} 2/2 Human Scout with haste. Landfall: it gains
/// battle cry until end of turn.
pub fn reckless_pyrosurfer() -> CardDefinition {
    CardDefinition {
        name: "Reckless Pyrosurfer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::BattleCry(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── Eldrazi Spawn ─────────────────────────────────────────────────────────────

/// 0/1 colorless Eldrazi Spawn with "Sacrifice this token: Add {C}."
fn eldrazi_spawn() -> TokenDefinition {
    TokenDefinition {
        name: "Eldrazi Spawn".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Colorless(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spawn-Gang Commander — {3}{R}{R} 2/2 devoid Eldrazi Goblin. When you cast it,
/// create three Eldrazi Spawn. {1}{C}, Sacrifice an Eldrazi: 2 damage to any
/// target.
pub fn spawn_gang_commander() -> CardDefinition {
    CardDefinition {
        name: "Spawn-Gang Commander",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![on_cast(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: eldrazi_spawn(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), colorless(1)]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Eldrazi), 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Eldrazi / big bodies ──────────────────────────────────────────────────────

/// Vaultborn Tyrant — {5}{G}{G} 6/6 Dinosaur with trample. Whenever this or
/// another creature you control with power 4+ enters, gain 3 life and draw.
/// When it dies (if nontoken), create a token copy that's also an artifact.
pub fn vaultborn_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Vaultborn Tyrant",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::PowerAtLeast(4)),
                    }),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(3) },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            },
            on_dies(Effect::If {
                cond: Predicate::EntityMatches { what: Selector::This, filter: R::NotToken },
                then: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![CardType::Artifact],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..Default::default()
    }
}

// ── Modified-matters ──────────────────────────────────────────────────────────

/// Hydra Trainer — {1}{G} 1/1 Human Warrior. Exert as it attacks: target
/// creature gets +X/+X, where X is the number of counters on permanents you
/// control. {2}{G}: Adapt 2.
pub fn hydra_trainer() -> CardDefinition {
    CardDefinition {
        name: "Hydra Trainer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::TotalCountersOn { what: Box::new(Selector::EachPermanent(R::ControlledByYou)) },
            toughness: Value::TotalCountersOn { what: Box::new(Selector::EachPermanent(R::ControlledByYou)) },
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: adapt(2),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Signature Slam — {2}{G} Instant. Put a +1/+1 counter on target creature you
/// control, then each modified creature you control deals damage equal to its
/// power to target creature you don't control.
pub fn signature_slam() -> CardDefinition {
    CardDefinition {
        name: "Signature Slam",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::IsModified)),
                body: Box::new(Effect::DealDamageEqualToPower {
                    source: Selector::TriggerSource,
                    target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                }),
            },
        ]),
        ..Default::default()
    }
}

// ── Artifact / recursion ──────────────────────────────────────────────────────

fn phyrexian_wurm(power: i32, toughness: i32, kw: Keyword) -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Wurm".into(),
        power,
        toughness,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        keywords: vec![kw],
        ..Default::default()
    }
}

/// Wurmcoil Larva — {3}{B}{B} 3/3 Artifact Creature — Phyrexian Wurm with
/// deathtouch and lifelink. When it dies, create a 1/2 deathtouch token and a
/// 2/1 lifelink token (both black Phyrexian Wurm artifact creatures).
pub fn wurmcoil_larva() -> CardDefinition {
    CardDefinition {
        name: "Wurmcoil Larva",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: phyrexian_wurm(1, 2, Keyword::Deathtouch),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: phyrexian_wurm(2, 1, Keyword::Lifelink),
            },
        ]))],
        ..Default::default()
    }
}

// ── Saga ──────────────────────────────────────────────────────────────────────

/// Cat Warrior token — 2/1 white.
fn cat_warrior() -> TokenDefinition {
    TokenDefinition {
        name: "Cat Warrior".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Ajani Fells the Godsire — {3}{W}{W} Saga. I — exile target opponent creature
/// with power 3+. II — make a 2/1 Cat Warrior and put a vigilance counter on a
/// creature you control. III — target creature you control gains double strike.
pub fn ajani_fells_the_godsire() -> CardDefinition {
    CardDefinition {
        name: "Ajani Fells the Godsire",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, Effect::Exile {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerAtLeast(3))),
            }),
            (2, Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: cat_warrior() },
                Effect::AddKeywordCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Vigilance,
                    amount: Value::ONE,
                },
            ])),
            (3, Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            }),
        ],
        ..Default::default()
    }
}
