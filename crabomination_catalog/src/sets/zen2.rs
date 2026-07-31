//! Zendikar (ZEN) gap closure — Traps, Allies, landfall and kicker.
//! Tests in `classic_sets/zen2`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    LandType, Predicate, SelectionRequirement as R, SpellSubtype, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{etb, landfall, mint_token, rally, target_any, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
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

fn ally(
    name: &'static str,
    c: crate::mana::ManaCost,
    mut types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    types.push(CreatureType::Ally);
    creature(name, c, types, p, t)
}

fn aura(name: &'static str, c: crate::mana::ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn ally_count() -> Value {
    Value::count(Selector::EachPermanent(
        R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
    ))
}

/// A Trap: "If [condition], you may pay [cost] rather than pay this spell's
/// mana cost."
fn trap(
    name: &'static str,
    printed: crate::mana::ManaCost,
    alt: crate::mana::ManaCost,
    condition: Predicate,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: printed,
        card_types: vec![CardType::Instant],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Trap], ..Default::default() },
        alternative_cost: Some(AlternativeCost {
            mana_cost: alt,
            condition: Some(condition),
            ..Default::default()
        }),
        effect,
        ..Default::default()
    }
}

/// Attacking creatures, for the "if N creatures are attacking" Trap gates.
fn attackers() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::IsAttacking))
}

// ── Traps ───────────────────────────────────────────────────────────────────

/// Arrow Volley Trap — {3}{W}{W}. {1}{W} with four attackers. 5 damage divided
/// among any number of target attacking creatures.
pub fn arrow_volley_trap() -> CardDefinition {
    trap(
        "Arrow Volley Trap",
        cost(&[generic(3), w(), w()]),
        cost(&[generic(1), w()]),
        Predicate::SelectorCountAtLeast { sel: attackers(), n: Value::Const(4) },
        Effect::DealDamageDivided {
            total: Value::Const(5),
            filter: R::Creature.and(R::IsAttacking),
            max_targets: 5,
            retaliate_to_source: false,
        },
    )
}

/// Pitfall Trap — {2}{W}. {W} with exactly one attacker. Destroy target
/// attacking creature without flying.
pub fn pitfall_trap() -> CardDefinition {
    trap(
        "Pitfall Trap",
        cost(&[generic(2), w()]),
        cost(&[w()]),
        Predicate::ValueEquals(Value::count(attackers()), Value::Const(1)),
        Effect::Destroy {
            what: target_filtered(
                R::Creature.and(R::IsAttacking).and(R::Not(Box::new(R::HasKeyword(
                    Keyword::Flying,
                )))),
            ),
        },
    )
}

/// Lethargy Trap — {3}{U}. {U} with three attackers. Attacking creatures get
/// −3/−0.
pub fn lethargy_trap() -> CardDefinition {
    trap(
        "Lethargy Trap",
        cost(&[generic(3), u()]),
        cost(&[u()]),
        Predicate::SelectorCountAtLeast { sel: attackers(), n: Value::Const(3) },
        Effect::PumpPT {
            what: attackers(),
            power: Value::Const(-3),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Mindbreak Trap — {2}{U}{U}. Free if an opponent cast three or more spells
/// this turn. Exile any number of target spells.
pub fn mindbreak_trap() -> CardDefinition {
    trap(
        "Mindbreak Trap",
        cost(&[generic(2), u(), u()]),
        cost(&[]),
        Predicate::SpellsCastThisTurnAtLeast {
            who: PlayerRef::EachOpponent,
            at_least: Value::Const(3),
        },
        Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 1,
            filter: R::IsSpellOnStack,
            effect: Box::new(Effect::CounterSpellToZone {
                what: Selector::Target(0),
                zone: crate::effect::CounteredSpellZone::Exile,
            }),
        },
    )
}

/// Inferno Trap — {3}{R}. {R} once two creatures have hit you this turn. 4
/// damage to target creature.
pub fn inferno_trap() -> CardDefinition {
    trap(
        "Inferno Trap",
        cost(&[generic(3), r()]),
        cost(&[r()]),
        Predicate::DamagedByCreaturesThisTurnAtLeast { who: PlayerRef::You, at_least: 2 },
        Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
    )
}

/// Lavaball Trap — {6}{R}{R}. {3}{R}{R} once an opponent has landed two lands
/// this turn. Destroy two lands and sweep 4 damage across creatures.
pub fn lavaball_trap() -> CardDefinition {
    trap(
        "Lavaball Trap",
        cost(&[generic(6), r(), r()]),
        cost(&[generic(3), r(), r()]),
        Predicate::LandsEnteredThisTurnAtLeast { who: PlayerRef::EachOpponent, at_least: 2 },
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Land,
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::Const(4),
            },
        ]),
    )
}

/// Baloth Cage Trap — {3}{G}{G}. {1}{G} once an opponent has landed an
/// artifact this turn. Make a 4/4 Beast.
pub fn baloth_cage_trap() -> CardDefinition {
    trap(
        "Baloth Cage Trap",
        cost(&[generic(3), g(), g()]),
        cost(&[generic(1), g()]),
        Predicate::ArtifactEnteredThisTurn { who: PlayerRef::EachOpponent },
        mint_token(
            TokenDefinition {
                name: "Beast".into(),
                power: 4,
                toughness: 4,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Beast],
                    ..Default::default()
                },
                ..Default::default()
            },
            1,
        ),
    )
}

/// Needlebite Trap — {5}{B}{B}. {B} once an opponent has gained life. Drain 5.
pub fn needlebite_trap() -> CardDefinition {
    trap(
        "Needlebite Trap",
        cost(&[generic(5), b(), b()]),
        cost(&[b()]),
        Predicate::PlayerGainedLifeThisTurn { who: PlayerRef::EachOpponent },
        Effect::Drain {
            from: Selector::Player(PlayerRef::Target(0)),
            to: Selector::You,
            amount: Value::Const(5),
        },
    )
}

// ── Allies ──────────────────────────────────────────────────────────────────

/// The "Rally — put a +1/+1 counter on this" Allies (Kazandu Blademaster,
/// Makindi Shieldmate, Nimana Sell-Sword, Oran-Rief Survivalist).
fn rally_self_counter(base: CardDefinition) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Put a +1/+1 counter on this creature".into(),
            body: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..base
    }
}

/// Kazandu Blademaster — {W}{W} 1/1 Human Soldier Ally with first strike and
/// vigilance; Rally grows it.
pub fn kazandu_blademaster() -> CardDefinition {
    rally_self_counter(CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        ..ally(
            "Kazandu Blademaster",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    })
}

/// Makindi Shieldmate — {2}{W} 0/3 Kor Soldier Ally with defender; Rally grows
/// it.
pub fn makindi_shieldmate() -> CardDefinition {
    rally_self_counter(CardDefinition {
        keywords: vec![Keyword::Defender],
        ..ally(
            "Makindi Shieldmate",
            cost(&[generic(2), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            0,
            3,
        )
    })
}

/// Nimana Sell-Sword — {3}{B} 2/2 Human Warrior Ally; Rally grows it.
pub fn nimana_sell_sword() -> CardDefinition {
    rally_self_counter(ally(
        "Nimana Sell-Sword",
        cost(&[generic(3), b()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        2,
        2,
    ))
}

/// Oran-Rief Survivalist — {1}{G} 1/1 Human Warrior Ally; Rally grows it.
pub fn oran_rief_survivalist() -> CardDefinition {
    rally_self_counter(ally(
        "Oran-Rief Survivalist",
        cost(&[generic(1), g()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        1,
        1,
    ))
}

/// Kazuul Warlord — {4}{R} 3/3 Minotaur Warrior Ally; Rally grows every Ally.
pub fn kazuul_warlord() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Put a +1/+1 counter on each Ally you control".into(),
            body: Box::new(Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..ally(
            "Kazuul Warlord",
            cost(&[generic(4), r()]),
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// The "Rally — your Allies gain <keyword>" commons (Highland Berserker,
/// Joraga Bard).
fn rally_grant_allies(base: CardDefinition, keyword: Keyword, label: &'static str) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: label.into(),
            body: Box::new(Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                ),
                keyword,
                duration: Duration::EndOfTurn,
            }),
        })],
        ..base
    }
}

/// Highland Berserker — {1}{R} 2/1 Human Berserker Ally; Rally grants first
/// strike.
pub fn highland_berserker() -> CardDefinition {
    rally_grant_allies(
        ally(
            "Highland Berserker",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            2,
            1,
        ),
        Keyword::FirstStrike,
        "Allies you control gain first strike",
    )
}

/// Joraga Bard — {3}{G} 1/4 Elf Rogue Bard Ally; Rally grants vigilance.
pub fn joraga_bard() -> CardDefinition {
    rally_grant_allies(
        ally(
            "Joraga Bard",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Rogue, CreatureType::Bard],
            1,
            4,
        ),
        Keyword::Vigilance,
        "Allies you control gain vigilance",
    )
}

/// Bala Ged Thief — {3}{B} 2/2 Human Rogue Ally; Rally strips a card from a
/// hand, one revealed per Ally.
pub fn bala_ged_thief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::DiscardChosenFromRevealed {
            from: Selector::Player(PlayerRef::Target(0)),
            reveal: ally_count(),
        })],
        ..ally(
            "Bala Ged Thief",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Hagra Diabolist — {4}{B} 3/2 Ogre Shaman Ally; Rally drains for your Ally
/// count.
pub fn hagra_diabolist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Target player loses life equal to your Ally count".into(),
            body: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: ally_count(),
            }),
        })],
        ..ally(
            "Hagra Diabolist",
            cost(&[generic(4), b()]),
            vec![CreatureType::Ogre, CreatureType::Shaman],
            3,
            2,
        )
    }
}

/// Murasa Pyromancer — {4}{R}{R} 3/2 Human Shaman Ally; Rally shoots a creature
/// for your Ally count.
pub fn murasa_pyromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Deal damage equal to your Ally count to target creature".into(),
            body: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: ally_count(),
            }),
        })],
        ..ally(
            "Murasa Pyromancer",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            3,
            2,
        )
    }
}

/// Kabira Evangel — {2}{W} 2/3 Human Cleric Ally; Rally gives your Allies
/// protection from a chosen color.
pub fn kabira_evangel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Allies you control gain protection from a color".into(),
            body: Box::new(Effect::GrantProtectionFromChosenColor {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..ally(
            "Kabira Evangel",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            3,
        )
    }
}

// ── Landfall ────────────────────────────────────────────────────────────────

/// Hagra Crocodile — {3}{B} 3/1 Crocodile that can't block; landfall pumps it.
pub fn hagra_crocodile() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![landfall(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Hagra Crocodile", cost(&[generic(3), b()]), vec![CreatureType::Crocodile], 3, 1)
    }
}

/// Hedron Scrabbler — {2} 1/1 Construct; landfall pumps it.
pub fn hedron_scrabbler() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![landfall(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Hedron Scrabbler", cost(&[generic(2)]), vec![CreatureType::Construct], 1, 1)
    }
}

/// Geyser Glider — {3}{R}{R} 4/4 Elemental Beast; landfall gives it flying.
pub fn geyser_glider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Geyser Glider",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Beast],
            4,
            4,
        )
    }
}

/// Ob Nixilis, the Fallen — {3}{B}{B} 3/3 Demon; landfall drains 3 and grows
/// him by three counters.
pub fn ob_nixilis_the_fallen() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Target player loses 3 life".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(3),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
            ])),
        })],
        ..creature("Ob Nixilis, the Fallen", cost(&[generic(3), b(), b()]), vec![CreatureType::Demon], 3, 3)
    }
}

/// Ior Ruin Expedition — {1}{U} Enchantment. Landfall banks quest counters;
/// cash three in for two cards.
pub fn ior_ruin_expedition() -> CardDefinition {
    CardDefinition {
        name: "Ior Ruin Expedition",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Put a quest counter on Ior Ruin Expedition".into(),
            body: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Quest,
                amount: Value::Const(1),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Quest, 3)),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Kicker ──────────────────────────────────────────────────────────────────

/// Goblin Ruinblaster — {2}{R} 2/1 Goblin Shaman with haste and kicker {R};
/// kicked, it blows up a nonbasic land.
pub fn goblin_ruinblaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Kicker(cost(&[r()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Destroy { what: target_filtered(R::IsNonbasicLand) }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Goblin Ruinblaster",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            2,
            1,
        )
    }
}

/// Heartstabber Mosquito — {3}{B} 2/2 Insect with flying and kicker {2}{B};
/// kicked, it kills a creature.
pub fn heartstabber_mosquito() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(2), b()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Destroy { what: target_filtered(R::Creature) }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Heartstabber Mosquito", cost(&[generic(3), b()]), vec![CreatureType::Insect], 2, 2)
    }
}

/// Mold Shambler — {3}{G} 3/3 Fungus Beast with kicker {1}{G}; kicked, it
/// destroys a noncreature permanent.
pub fn mold_shambler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), g()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Destroy {
                what: target_filtered(R::Permanent.and(R::Noncreature)),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Mold Shambler",
            cost(&[generic(3), g()]),
            vec![CreatureType::Fungus, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Oran-Rief Recluse — {2}{G} 1/3 Spider with reach and kicker {2}{G}; kicked,
/// it shoots down a flier.
pub fn oran_rief_recluse() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Kicker(cost(&[generic(2), g()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Oran-Rief Recluse", cost(&[generic(2), g()]), vec![CreatureType::Spider], 1, 3)
    }
}

/// Kor Aeronaut — {W}{W} 2/2 Kor Soldier with flying and kicker {1}{W}; kicked,
/// it lends flying to a creature.
pub fn kor_aeronaut() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(1), w()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Kor Aeronaut",
            cost(&[w(), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Bold Defense — {2}{W} Instant with kicker {3}{W}. +1/+1, or +2/+2 and first
/// strike when kicked.
pub fn bold_defense() -> CardDefinition {
    let pump = |n: i32| Effect::PumpPT {
        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
        power: Value::Const(n),
        toughness: Value::Const(n),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Bold Defense",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(3), w()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                pump(2),
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ])),
            else_: Box::new(pump(1)),
        },
        ..Default::default()
    }
}

/// Conqueror's Pledge — {2}{W}{W}{W} Sorcery with kicker {6}. Six Kor Soldiers,
/// or twelve when kicked.
pub fn conquerors_pledge() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Kor Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Conqueror's Pledge",
        cost: cost(&[generic(2), w(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(6)]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(mint_token(soldier.clone(), 12)),
            else_: Box::new(mint_token(soldier, 6)),
        },
        ..Default::default()
    }
}

/// Elemental Appeal — {R}{R}{R}{R} Sorcery with kicker {5}. A 7/1 trampling
/// hasty Elemental for the turn, +7/+0 when kicked.
pub fn elemental_appeal() -> CardDefinition {
    CardDefinition {
        name: "Elemental Appeal",
        cost: cost(&[r(), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(5)]))],
        effect: Effect::Seq(vec![
            mint_token(
                TokenDefinition {
                    name: "Elemental".into(),
                    power: 7,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    keywords: vec![Keyword::Trample, Keyword::Haste],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Elemental],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                1,
            ),
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::PumpPT {
                    what: Selector::LastCreatedTokens,
                    power: Value::Const(7),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::ExileLastCreatedTokensAtNextEndStep,
        ]),
        ..Default::default()
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// The ZEN "Refuge" cycle: enters tapped, gains 1 life, taps for either of two
/// colors.
fn refuge(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![a]) },
            ..Default::default()
        },
        ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![b]) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Akoum Refuge — {B} or {R}.
pub fn akoum_refuge() -> CardDefinition {
    refuge("Akoum Refuge", Color::Black, Color::Red)
}

/// Graypelt Refuge — {G} or {W}.
pub fn graypelt_refuge() -> CardDefinition {
    refuge("Graypelt Refuge", Color::Green, Color::White)
}

/// Jwar Isle Refuge — {U} or {B}.
pub fn jwar_isle_refuge() -> CardDefinition {
    refuge("Jwar Isle Refuge", Color::Blue, Color::Black)
}

/// Kazandu Refuge — {R} or {G}.
pub fn kazandu_refuge() -> CardDefinition {
    refuge("Kazandu Refuge", Color::Red, Color::Green)
}

/// Kabira Crossroads — enters tapped, gains 2 life, taps for {W}.
pub fn kabira_crossroads() -> CardDefinition {
    super::wwk::tapped_etb_land(
        "Kabira Crossroads",
        Color::White,
        Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
    )
}

/// Piranha Marsh — enters tapped, drains 1, taps for {B}.
pub fn piranha_marsh() -> CardDefinition {
    super::wwk::tapped_etb_land(
        "Piranha Marsh",
        Color::Black,
        Effect::LoseLife {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(1),
        },
    )
}

// ── The rest ────────────────────────────────────────────────────────────────

/// Bladetusk Boar — {3}{R} 3/2 Boar with intimidate.
pub fn bladetusk_boar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Intimidate],
        ..creature("Bladetusk Boar", cost(&[generic(3), r()]), vec![CreatureType::Boar], 3, 2)
    }
}

/// Bog Tatters — {4}{B} 4/2 Wraith with swampwalk.
pub fn bog_tatters() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..creature("Bog Tatters", cost(&[generic(4), b()]), vec![CreatureType::Wraith], 4, 2)
    }
}

/// Cliff Threader — {1}{W} 2/1 Kor Scout with mountainwalk.
pub fn cliff_threader() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..creature(
            "Cliff Threader",
            cost(&[generic(1), w()]),
            vec![CreatureType::Kor, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Caravan Hurda — {4}{W} 1/5 Giant with lifelink.
pub fn caravan_hurda() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        ..creature("Caravan Hurda", cost(&[generic(4), w()]), vec![CreatureType::Giant], 1, 5)
    }
}

/// Giant Scorpion — {2}{B} 1/3 Scorpion with deathtouch.
pub fn giant_scorpion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        ..creature("Giant Scorpion", cost(&[generic(2), b()]), vec![CreatureType::Scorpion], 1, 3)
    }
}

/// Crypt Ripper — {2}{B}{B} 2/2 Shade with haste and a {B} pump.
pub fn crypt_ripper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Crypt Ripper", cost(&[generic(2), b(), b()]), vec![CreatureType::Shade], 2, 2)
    }
}

/// Molten Ravager — {2}{R} 0/4 Elemental with firebreathing.
pub fn molten_ravager() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Molten Ravager", cost(&[generic(2), r()]), vec![CreatureType::Elemental], 0, 4)
    }
}

/// Caller of Gales — {U} 1/1 Merfolk Wizard. {1}{U}, {T}: a creature gains
/// flying.
pub fn caller_of_gales() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Caller of Gales",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Reckless Scholar — {2}{U} 2/1 Human Wizard. {T}: a player loots.
pub fn reckless_scholar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(1),
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Reckless Scholar",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Frontier Guide — {1}{G} 1/1 Elf Scout. {3}{G}, {T}: fetch a basic tapped.
pub fn frontier_guide() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature(
            "Frontier Guide",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Kor Cartographer — {3}{W} 2/2 Kor Scout. ETB fetches a Plains tapped.
pub fn kor_cartographer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::HasLandType(LandType::Plains),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        })],
        ..creature(
            "Kor Cartographer",
            cost(&[generic(3), w()]),
            vec![CreatureType::Kor, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Goblin Shortcutter — {1}{R} 2/1 Goblin Scout. ETB stops a blocker.
pub fn goblin_shortcutter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Goblin Shortcutter",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Halo Hunter — {2}{B}{B}{B} 6/3 Demon with intimidate. ETB kills an Angel.
pub fn halo_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Intimidate],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::HasCreatureType(CreatureType::Angel)),
        })],
        ..creature("Halo Hunter", cost(&[generic(2), b(), b(), b()]), vec![CreatureType::Demon], 6, 3)
    }
}

/// Devout Lightcaster — {W}{W}{W} 2/2 Kor Cleric with protection from black.
/// ETB exiles a black permanent.
pub fn devout_lightcaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        triggered_abilities: vec![etb(Effect::Exile {
            what: target_filtered(R::Permanent.and(R::HasColor(Color::Black))),
        })],
        ..creature(
            "Devout Lightcaster",
            cost(&[w(), w(), w()]),
            vec![CreatureType::Kor, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Desecrated Earth — {4}{B} Sorcery. Destroy a land; its controller discards.
pub fn desecrated_earth() -> CardDefinition {
    CardDefinition {
        name: "Desecrated Earth",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Land) },
            Effect::Discard {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(1),
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Narrow Escape — {2}{W} Instant. Bounce your own permanent and gain 4.
pub fn narrow_escape() -> CardDefinition {
    CardDefinition {
        name: "Narrow Escape",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

/// Landbind Ritual — {3}{W}{W} Sorcery. Gain 2 life per Plains.
pub fn landbind_ritual() -> CardDefinition {
    CardDefinition {
        name: "Landbind Ritual",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::count(Selector::EachPermanent(
                    R::HasLandType(LandType::Plains).and(R::ControlledByYou),
                ))),
            ),
        },
        ..Default::default()
    }
}

/// Primal Bellow — {G} Instant. +1/+1 per Forest you control.
pub fn primal_bellow() -> CardDefinition {
    CardDefinition {
        name: "Primal Bellow",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::count(Selector::EachPermanent(
                R::HasLandType(LandType::Forest).and(R::ControlledByYou),
            )),
            toughness: Value::count(Selector::EachPermanent(
                R::HasLandType(LandType::Forest).and(R::ControlledByYou),
            )),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Magma Rift — {2}{R} Sorcery. Sacrifice a land; 5 damage to a creature.
pub fn magma_rift() -> CardDefinition {
    CardDefinition {
        name: "Magma Rift",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}

/// Relic Crush — {4}{G} Instant. Break up to two artifacts and/or enchantments.
pub fn relic_crush() -> CardDefinition {
    CardDefinition {
        name: "Relic Crush",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 1,
            filter: R::Artifact.or(R::Enchantment),
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

/// Feast of Blood — {1}{B} Sorcery, castable only with two Vampires. Kill a
/// creature and gain 4.
pub fn feast_of_blood() -> CardDefinition {
    CardDefinition {
        name: "Feast of Blood",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        cast_condition: Some(Predicate::SelectorCountAtLeast {
            sel: Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Vampire).and(R::ControlledByYou),
            ),
            n: Value::Const(2),
        }),
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

/// Carnage Altar — {2} Artifact. {3}, sacrifice a creature: draw a card.
pub fn carnage_altar() -> CardDefinition {
    CardDefinition {
        name: "Carnage Altar",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Khalni Gem — {4} Artifact. ETB bounces two of your lands; taps for two of
/// one color.
pub fn khalni_gem() -> CardDefinition {
    CardDefinition {
        name: "Khalni Gem",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::MoveChosen {
            from: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            filter: None,
            count: Value::Const(2),
            up_to: false,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blazing Torch — {1} Equipment. The bearer dodges Vampires and Zombies and
/// can throw the Torch for 2. Equip {1}.
pub fn blazing_torch() -> CardDefinition {
    CardDefinition {
        name: "Blazing Torch",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantBeBlockedBy(Box::new(
                R::HasCreatureType(CreatureType::Vampire)
                    .or(R::HasCreatureType(CreatureType::Zombie)),
            ))],
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::Const(1),
                        filter: R::HasName("Blazing Torch".into()),
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Goblin War Paint — {1}{R} Aura. +2/+2 and haste.
pub fn goblin_war_paint() -> CardDefinition {
    aura(
        "Goblin War Paint",
        cost(&[generic(1), r()]),
        EquipBonus { power: 2, toughness: 2, keywords: vec![Keyword::Haste], ..Default::default() },
    )
}

/// Nimbus Wings — {1}{W} Aura. +1/+2 and flying.
pub fn nimbus_wings() -> CardDefinition {
    aura(
        "Nimbus Wings",
        cost(&[generic(1), w()]),
        EquipBonus {
            power: 1,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        },
    )
}

/// Mire Blight — {B} Aura. Any damage destroys the enchanted creature.
pub fn mire_blight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::EnchantedBySource),
            effect: Effect::Destroy {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..aura("Mire Blight", cost(&[b()]), EquipBonus::default())
    }
}

/// Nissa's Chosen — {G}{G} 2/3 Elf Warrior that goes to the bottom of the
/// library instead of dying.
pub fn nissas_chosen() -> CardDefinition {
    CardDefinition {
        dies_to_library_bottom: true,
        ..creature(
            "Nissa's Chosen",
            cost(&[g(), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Mindless Null — {2}{B} 2/2 Zombie that can't block without a Vampire.
pub fn mindless_null() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Can't block unless you control a Vampire",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::CantBlock,
                condition: Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Vampire).and(R::ControlledByYou),
                    ),
                ))),
            },
        }],
        ..creature("Mindless Null", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Guul Draz Specter — {2}{B}{B} 2/2 Specter with flying, +3/+3 while an
/// opponent is hellbent, and a discard-on-connect rider.
pub fn guul_draz_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "+3/+3 as long as an opponent has no cards in hand",
            effect: StaticEffect::PumpSelfIf {
                power: 3,
                toughness: 3,
                condition: Predicate::HellbentActive { who: PlayerRef::EachOpponent },
                keywords: vec![],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: false,
            },
        }],
        ..creature("Guul Draz Specter", cost(&[generic(2), b(), b()]), vec![CreatureType::Specter], 2, 2)
    }
}

/// Hellfire Mongrel — {2}{R} 2/2 Elemental Dog. Each opponent's upkeep, burn
/// them for 2 while they're at two or fewer cards.
pub fn hellfire_mongrel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::OpponentControl,
            )
            .with_filter(Predicate::ValueAtMost(
                Value::HandSizeOf(PlayerRef::ActivePlayer),
                Value::Const(2),
            )),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(2),
            },
        }],
        ..creature(
            "Hellfire Mongrel",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental, CreatureType::Dog],
            2,
            2,
        )
    }
}

/// Armament Master — {W}{W} 2/2 Kor Soldier. Other Kor get +2/+2 per Equipment
/// on this creature.
pub fn armament_master() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Kor get +2/+2 for each Equipment attached to this",
            effect: StaticEffect::PumpTeamPerAttachmentOnSource {
                applies_to: R::HasCreatureType(CreatureType::Kor).and(R::OtherThanSource),
                attachment_filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                per_power: 2,
                per_toughness: 2,
            },
        }],
        ..creature(
            "Armament Master",
            cost(&[w(), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            2,
            2,
        )
    }
}

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Blade of the Bloodchief — {1} Equipment. Every creature death grows the
/// bearer; a Vampire bearer grows twice as fast. Equip {1}.
pub fn blade_of_the_bloodchief() -> CardDefinition {
    let counters = |n: i32| Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    };
    CardDefinition {
        name: "Blade of the Bloodchief",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
                effect: Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::HasCreatureType(CreatureType::Vampire),
                    },
                    then: Box::new(counters(2)),
                    else_: Box::new(counters(1)),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Grappling Hook — {4} Equipment. Double strike, and the bearer picks its own
/// blocker. Equip {4}.
pub fn grappling_hook() -> CardDefinition {
    CardDefinition {
        name: "Grappling Hook",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::DoubleStrike],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Target creature blocks this creature if able".into(),
                    body: Box::new(Effect::MustBlockSource {
                        what: target_filtered(R::Creature),
                    }),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Explorer's Scope — {1} Equipment. Attacking turns a land off the top into a
/// tapped land drop. Equip {1}.
pub fn explorers_scope() -> CardDefinition {
    CardDefinition {
        name: "Explorer's Scope",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::RevealTopLandToBattlefieldElseHand { who: PlayerRef::You },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Celestial Mantle — {3}{W}{W}{W} Aura. +3/+3, and connecting doubles its
/// controller's life total.
pub fn celestial_mantle() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::DoubleLife { who: Selector::You },
            }],
            ..Default::default()
        }),
        ..aura("Celestial Mantle", cost(&[generic(3), w(), w(), w()]), EquipBonus::default())
    }
}

/// Savage Silhouette — {2}{G} Aura. +2/+2 and a {1}{G} regeneration.
pub fn savage_silhouette() -> CardDefinition {
    aura(
        "Savage Silhouette",
        cost(&[generic(2), g()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Predatory Urge — {3}{G} Aura. The enchanted creature can tap to fight.
pub fn predatory_urge() -> CardDefinition {
    aura(
        "Predatory Urge",
        cost(&[generic(3), g()]),
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Fight {
                    attacker: Selector::This,
                    defender: target_filtered(R::Creature),
                },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Cosi's Trickster — {U} 1/1 Merfolk Wizard that grows whenever an opponent
/// shuffles.
pub fn cosis_trickster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LibraryShuffled, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Cosi's Trickster".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
        }],
        ..creature(
            "Cosi's Trickster",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Scute Mob — {G} 1/1 Insect that jumps four counters each upkeep once you
/// have five lands.
pub fn scute_mob() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                n: Value::Const(5),
            }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            },
        }],
        ..creature("Scute Mob", cost(&[g()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Scythe Tiger — {G} 3/2 Cat with shroud; it eats a land on the way in or
/// dies.
pub fn scythe_tiger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                R::Land.and(R::ControlledByYou),
            )),
            then: Box::new(Effect::MayDoElse {
                description: "Sacrifice a land".into(),
                body: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: R::Land,
                }),
                else_: Box::new(Effect::SacrificeSource),
            }),
            else_: Box::new(Effect::SacrificeSource),
        })],
        ..creature("Scythe Tiger", cost(&[g()]), vec![CreatureType::Cat], 3, 2)
    }
}

/// Living Tsunami — {2}{U}{U} 4/4 Elemental with flying; each upkeep it wants a
/// land back in your hand or it dies.
pub fn living_tsunami() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Land.and(R::ControlledByYou),
                )),
                then: Box::new(Effect::MayDoElse {
                    description: "Return a land you control to its owner's hand".into(),
                    body: Box::new(Effect::MoveChosen {
                        from: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                        filter: None,
                        count: Value::Const(1),
                        up_to: false,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: Box::new(Effect::SacrificeSource),
                }),
                else_: Box::new(Effect::SacrificeSource),
            },
        }],
        ..creature("Living Tsunami", cost(&[generic(2), u(), u()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Merfolk Seastalkers — {3}{U} 2/3 Merfolk Scout with islandwalk and a
/// {2}{U} ground-tapper.
pub fn merfolk_seastalkers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Tap {
                what: target_filtered(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Merfolk Seastalkers",
            cost(&[generic(3), u()]),
            vec![CreatureType::Merfolk, CreatureType::Scout],
            2,
            3,
        )
    }
}

/// Merfolk Wayfinder — {2}{U} 1/2 Merfolk Scout with flying. ETB digs three for
/// Islands.
pub fn merfolk_wayfinder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::RevealTopTakeMatchingToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            filter: R::HasLandType(LandType::Island),
        })],
        ..creature(
            "Merfolk Wayfinder",
            cost(&[generic(2), u()]),
            vec![CreatureType::Merfolk, CreatureType::Scout],
            1,
            2,
        )
    }
}

/// Sea Gate Loremaster — {4}{U} 1/3 Merfolk Wizard Ally. {T}: draw one card per
/// Ally.
pub fn sea_gate_loremaster() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: ally_count() },
            ..Default::default()
        }],
        ..ally(
            "Sea Gate Loremaster",
            cost(&[generic(4), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Seascape Aerialist — {4}{U} 2/3 Merfolk Wizard Ally; Rally grants flying.
pub fn seascape_aerialist() -> CardDefinition {
    rally_grant_allies(
        ally(
            "Seascape Aerialist",
            cost(&[generic(4), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            3,
        ),
        Keyword::Flying,
        "Allies you control gain flying",
    )
}

/// Noble Vestige — {2}{W} 1/2 Spirit with flying. {T}: shield a player for 1.
pub fn noble_vestige() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..creature("Noble Vestige", cost(&[generic(2), w()]), vec![CreatureType::Spirit], 1, 2)
    }
}

/// Ruinous Minotaur — {1}{R}{R} 5/2 Minotaur Warrior that eats one of your
/// lands whenever it connects.
pub fn ruinous_minotaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: R::Land,
            },
        }],
        ..creature(
            "Ruinous Minotaur",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            5,
            2,
        )
    }
}

/// Hellkite Charger — {4}{R}{R} 5/5 Dragon with flying and haste. Attacking, it
/// buys an extra combat for {5}{R}{R}.
pub fn hellkite_charger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {5}{R}{R} for an additional combat phase".into(),
                mana_cost: cost(&[generic(5), r(), r()]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Untap { what: attackers(), up_to: None },
                    Effect::AdditionalCombatPhase { count: Value::Const(1) },
                ])),
                else_: None,
            },
        }],
        ..creature("Hellkite Charger", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Lorthos, the Tidemaker — {5}{U}{U}{U} 8/8 Octopus. Attacking, {8} locks down
/// eight permanents.
pub fn lorthos_the_tidemaker() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {8} to tap up to eight permanents".into(),
                mana_cost: cost(&[generic(8)]),
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Permanent,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::Target(0) },
                        Effect::SkipNextUntap { what: Selector::Target(0) },
                    ])),
                }),
                else_: None,
            },
        }],
        ..creature("Lorthos, the Tidemaker", cost(&[generic(5), u(), u(), u()]), vec![CreatureType::Octopus], 8, 8)
    }
}

/// Electropotence — {2}{R} Enchantment. Each of your creatures entering may pay
/// {2}{R} to shoot for its power.
pub fn electropotence() -> CardDefinition {
    CardDefinition {
        name: "Electropotence",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::MayPay {
                description: "Pay {2}{R} to have it deal damage equal to its power".into(),
                mana_cost: cost(&[generic(2), r()]),
                body: Box::new(Effect::DealDamage {
                    to: target_any(),
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Eldrazi Monument — {5} Artifact. Your team flies and is indestructible, but
/// it eats a creature each upkeep.
pub fn eldrazi_monument() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Eldrazi Monument",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +1/+1",
                effect: StaticEffect::PumpPT { applies_to: team(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Creatures you control have flying",
                effect: StaticEffect::GrantKeyword {
                    applies_to: team(),
                    keyword: Keyword::Flying,
                },
            },
            StaticAbility {
                description: "Creatures you control have indestructible",
                effect: StaticEffect::GrantKeyword {
                    applies_to: team(),
                    keyword: Keyword::Indestructible,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorExists(team()),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: R::Creature,
                }),
                else_: Box::new(Effect::SacrificeSource),
            },
        }],
        ..Default::default()
    }
}

/// Sadistic Sacrament — {B}{B}{B} Sorcery with kicker {7}. Strip three cards
/// out of a library, or fifteen when kicked.
pub fn sadistic_sacrament() -> CardDefinition {
    let strip = |n: i32| Effect::Repeat {
        count: Value::Const(n),
        body: Box::new(Effect::SearchPickedBy {
            who: PlayerRef::Target(0),
            picker: PlayerRef::You,
            filter: R::Any,
            to: ZoneDest::Exile,
        }),
    };
    CardDefinition {
        name: "Sadistic Sacrament",
        cost: cost(&[b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(7)]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(strip(15)),
            else_: Box::new(strip(3)),
        },
        ..Default::default()
    }
}

/// Grim Discovery — {1}{B} Sorcery. Choose one or both: a creature card and/or
/// a land card back from your graveyard.
pub fn grim_discovery() -> CardDefinition {
    let back = |filter: R, slot: u8| Effect::Move {
        what: Selector::TargetFiltered { slot, filter: filter.and(R::InYourGraveyard) },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Grim Discovery",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![back(R::Creature, 0), back(R::Land, 0)],
        },
        ..Default::default()
    }
}

/// Crypt of Agadeem — enters tapped, taps for {B}; {2},{T} taps for {B} per
/// black creature card in your graveyard.
pub fn crypt_of_agadeem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Black]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(
                        Color::Black,
                        Value::count(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Graveyard,
                            filter: R::Creature.and(R::HasColor(Color::Black)),
                        }),
                    ),
                },
                ..Default::default()
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        name: "Crypt of Agadeem",
        card_types: vec![CardType::Land],
        ..Default::default()
    }
}

/// Emeria, the Sky Ruin — enters tapped, taps for {W}; each upkeep with seven
/// Plains it reanimates.
pub fn emeria_the_sky_ruin() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::HasLandType(LandType::Plains).and(R::ControlledByYou),
                ),
                n: Value::Const(7),
            }),
            effect: Effect::MayDo {
                description: "Return a creature card from your graveyard to the battlefield".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..super::wwk::tapped_etb_land("Emeria, the Sky Ruin", Color::White, Effect::Noop)
    }
}
