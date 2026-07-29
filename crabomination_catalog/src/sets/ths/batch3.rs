//! Theros (THS) — batch 3: the devotion / monstrosity / heroic / bestow tail
//! plus the gold uncommons and the remaining spells. Tests in
//! `classic_sets/ths`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, StaticAbility, StaticEffect,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    etb, heroic, monstrosity, on_attack, on_becomes_monstrous, target_any, target_filtered,
};
use crate::effect::{Duration, Effect, Predicate, PlayerRef, Selector, ZoneDest};
use crate::effect::ManaPayload;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

fn creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: ct, ..Default::default() },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
}

/// An enchantment creature with bestow granting `+p/+t` and `kw`.
fn bestow_creature(
    name: &'static str,
    mana: ManaCost,
    bestow_cost: ManaCost,
    pt: (i32, i32),
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
    bonus_pt: (i32, i32),
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(bestow_cost),
        equipped_bonus: Some(EquipBonus {
            power: bonus_pt.0,
            toughness: bonus_pt.1,
            keywords: kw.clone(),
            ..Default::default()
        }),
        ..creature(name, mana, pt.0, pt.1, ct, kw)
    }
}

fn scry(n: i32) -> Effect {
    Effect::Scry { who: PlayerRef::You, amount: Value::Const(n) }
}

fn counters_on_self(n: i32) -> Effect {
    Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Silent Artisan — {3}{W}{W} 3/5 Giant.
pub fn silent_artisan() -> CardDefinition {
    creature("Silent Artisan", cost(&[generic(3), w(), w()]), 3, 5, vec![CreatureType::Giant], vec![])
}

/// Setessan Griffin — {4}{W} 3/2 Griffin with flying. {2}{G}{G}: +2/+2 until
/// end of turn (once each turn).
pub fn setessan_griffin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Setessan Griffin",
            cost(&[generic(4), w()]),
            3,
            2,
            vec![CreatureType::Griffin],
            vec![Keyword::Flying],
        )
    }
}

/// Setessan Battle Priest — {1}{W} 1/3 Human Cleric. Heroic: gain 2 life.
pub fn setessan_battle_priest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..creature(
            "Setessan Battle Priest",
            cost(&[generic(1), w()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Fabled Hero — {1}{W}{W} 2/2 Human Soldier with double strike. Heroic: a
/// +1/+1 counter.
pub fn fabled_hero() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(counters_on_self(1))],
        ..creature(
            "Fabled Hero",
            cost(&[generic(1), w(), w()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![Keyword::DoubleStrike],
        )
    }
}

/// Wingsteed Rider — {1}{W}{W} 2/2 Human Knight with flying. Heroic: a +1/+1
/// counter.
pub fn wingsteed_rider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(counters_on_self(1))],
        ..creature(
            "Wingsteed Rider",
            cost(&[generic(1), w(), w()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Knight],
            vec![Keyword::Flying],
        )
    }
}

/// Soldier of the Pantheon — {W} 2/1 Human Soldier with protection from
/// multicolored. Whenever an opponent casts a multicolored spell, gain 1 life.
pub fn soldier_of_the_pantheon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::CastSpellMatches(R::Multicolored)),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Soldier of the Pantheon",
            cost(&[w()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![Keyword::ProtectionFromMulticolored],
        )
    }
}

/// Evangel of Heliod — {4}{W}{W} 1/3 Human Cleric. ETB: create a 1/1 white
/// Soldier for each point of your devotion to white.
pub fn evangel_of_heliod() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::DevotionTo(vec![Color::White]),
            definition: TokenDefinition {
                name: "Soldier".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Soldier],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..creature(
            "Evangel of Heliod",
            cost(&[generic(4), w(), w()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Hundred-Handed One — {2}{W}{W} 3/5 Giant with vigilance. {3}{W}{W}{W}:
/// Monstrosity 3; while monstrous it has reach and blocks 99 extra creatures.
pub fn hundred_handed_one() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(3), w(), w(), w()]), 3)],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is monstrous, it has reach and can \
                          block an additional ninety-nine creatures each combat.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsMonstrous,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Reach, Keyword::CanBlockAdditional(99)],
            },
        }],
        ..creature(
            "Hundred-Handed One",
            cost(&[generic(2), w(), w()]),
            3,
            5,
            vec![CreatureType::Giant],
            vec![Keyword::Vigilance],
        )
    }
}

/// Scholar of Athreos — {2}{W} 1/4 Human Cleric. {2}{B}: each opponent loses 1
/// life and you gain that much.
pub fn scholar_of_athreos() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Scholar of Athreos",
            cost(&[generic(2), w()]),
            1,
            4,
            vec![CreatureType::Human, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Ray of Dissolution — {2}{W} Instant. Destroy target enchantment; gain 3 life.
pub fn ray_of_dissolution() -> CardDefinition {
    spell(
        "Ray of Dissolution",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]),
    )
}

/// Vanquish the Foul — {5}{W} Sorcery. Destroy target creature with power 4 or
/// greater. Scry 1.
pub fn vanquish_the_foul() -> CardDefinition {
    spell(
        "Vanquish the Foul",
        cost(&[generic(5), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
            scry(1),
        ]),
    )
}

/// Celestial Archon — {3}{W}{W} 4/4 Archon with flying and first strike.
/// Bestow {5}{W}{W}: +4/+4 and both keywords.
pub fn celestial_archon() -> CardDefinition {
    bestow_creature(
        "Celestial Archon",
        cost(&[generic(3), w(), w()]),
        cost(&[generic(5), w(), w()]),
        (4, 4),
        vec![CreatureType::Archon],
        vec![Keyword::Flying, Keyword::FirstStrike],
        (4, 4),
    )
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Triton Shorethief — {U} 1/2 Merfolk Rogue.
pub fn triton_shorethief() -> CardDefinition {
    creature(
        "Triton Shorethief",
        cost(&[u()]),
        1,
        2,
        vec![CreatureType::Merfolk, CreatureType::Rogue],
        vec![],
    )
}

/// Vaporkin — {1}{U} 2/1 Elemental with flying that can block only creatures
/// with flying.
pub fn vaporkin() -> CardDefinition {
    creature(
        "Vaporkin",
        cost(&[generic(1), u()]),
        2,
        1,
        vec![CreatureType::Elemental],
        vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
    )
}

/// Prescient Chimera — {3}{U}{U} 3/4 Chimera with flying. Whenever you cast an
/// instant or sorcery spell, scry 1.
pub fn prescient_chimera() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))),
            effect: scry(1),
        }],
        ..creature(
            "Prescient Chimera",
            cost(&[generic(3), u(), u()]),
            3,
            4,
            vec![CreatureType::Chimera],
            vec![Keyword::Flying],
        )
    }
}

/// Prognostic Sphinx — {3}{U}{U} 3/5 Sphinx with flying. Discard a card: gains
/// hexproof until end of turn, then tap it. Whenever it attacks, scry 3.
pub fn prognostic_sphinx() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![on_attack(scry(3))],
        ..creature(
            "Prognostic Sphinx",
            cost(&[generic(3), u(), u()]),
            3,
            5,
            vec![CreatureType::Sphinx],
            vec![Keyword::Flying],
        )
    }
}

/// Triton Fortune Hunter — {2}{U} 2/2 Merfolk Soldier. Heroic: draw a card.
pub fn triton_fortune_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Triton Fortune Hunter",
            cost(&[generic(2), u()]),
            2,
            2,
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Wavecrash Triton — {2}{U} 1/4 Merfolk Wizard. Heroic: tap target creature an
/// opponent controls and stun it.
pub fn wavecrash_triton() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ]))],
        ..creature(
            "Wavecrash Triton",
            cost(&[generic(2), u()]),
            1,
            4,
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Sea God's Revenge — {5}{U} Sorcery. Return up to three target creatures your
/// opponents control to their owners' hands. Scry 1.
pub fn sea_gods_revenge() -> CardDefinition {
    spell(
        "Sea God's Revenge",
        cost(&[generic(5), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            scry(1),
        ]),
    )
}

/// Thassa's Bounty — {5}{U} Sorcery. Draw three cards; target player mills three.
pub fn thassas_bounty() -> CardDefinition {
    spell(
        "Thassa's Bounty",
        cost(&[generic(5), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(3) },
        ]),
    )
}

/// Stymied Hopes — {1}{U} Instant. Counter target spell unless its controller
/// pays {1}. Scry 1.
pub fn stymied_hopes() -> CardDefinition {
    spell(
        "Stymied Hopes",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
            },
            scry(1),
        ]),
    )
}

/// Thassa's Emissary — {3}{U} 3/3 Crab. Bestow {5}{U}: +3/+3. Whenever it or
/// the enchanted creature deals combat damage to a player, draw a card.
pub fn thassas_emissary() -> CardDefinition {
    let draw_on_damage = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
    };
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(cost(&[generic(5), u()])),
        triggered_abilities: vec![draw_on_damage.clone()],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            triggered_abilities: vec![draw_on_damage],
            ..Default::default()
        }),
        ..creature(
            "Thassa's Emissary",
            cost(&[generic(3), u()]),
            3,
            3,
            vec![CreatureType::Crab],
            vec![],
        )
    }
}

/// Sealock Monster — {3}{U}{U} 5/5 Octopus that can't attack unless the
/// defending player controls an Island. {5}{U}{U}: Monstrosity 3; when it
/// becomes monstrous, target land becomes an Island in addition to its types.
pub fn sealock_monster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::HasLandType(
            LandType::Island,
        )))],
        activated_abilities: vec![monstrosity(cost(&[generic(5), u(), u()]), 3)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::GainLandType {
            what: target_filtered(R::Land),
            land_type: LandType::Island,
            duration: Duration::Permanent,
        })],
        ..creature(
            "Sealock Monster",
            cost(&[generic(3), u(), u()]),
            5,
            5,
            vec![CreatureType::Octopus],
            vec![],
        )
    }
}

/// Meletis Charlatan — {2}{U} 2/3 Human Wizard. {2}{U}, {T}: the controller of
/// target instant or sorcery spell copies it and may choose new targets.
pub fn meletis_charlatan() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            effect: Effect::CopySpellMayChooseTargets {
                what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))),
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Meletis Charlatan",
            cost(&[generic(2), u()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Insatiable Harpy — {2}{B}{B} 2/2 Harpy with flying and lifelink.
pub fn insatiable_harpy() -> CardDefinition {
    creature(
        "Insatiable Harpy",
        cost(&[generic(2), b(), b()]),
        2,
        2,
        vec![CreatureType::Harpy],
        vec![Keyword::Flying, Keyword::Lifelink],
    )
}

/// Returned Phalanx — {1}{B} 3/3 Zombie Soldier with defender. {1}{U}: it can
/// attack this turn as though it didn't have defender.
pub fn returned_phalanx() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::AttackDespiteDefenderThisTurn { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Returned Phalanx",
            cost(&[generic(1), b()]),
            3,
            3,
            vec![CreatureType::Zombie, CreatureType::Soldier],
            vec![Keyword::Defender],
        )
    }
}

/// Tormented Hero — {B} 2/1 Human Warrior that enters tapped. Heroic: each
/// opponent loses 1 life and you gain that much.
pub fn tormented_hero() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![heroic(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Tormented Hero",
            cost(&[b()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Agent of the Fates — {1}{B}{B} 3/2 Human Assassin with deathtouch. Heroic:
/// each opponent sacrifices a creature.
pub fn agent_of_the_fates() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::ONE,
            filter: R::Creature,
        })],
        ..creature(
            "Agent of the Fates",
            cost(&[generic(1), b(), b()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Assassin],
            vec![Keyword::Deathtouch],
        )
    }
}

/// Disciple of Phenax — {2}{B}{B} 1/3 Human Cleric. ETB: target player reveals
/// cards equal to your devotion to black; you choose one to be discarded.
pub fn disciple_of_phenax() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DiscardChosenFromRevealed {
            from: Selector::Player(PlayerRef::Target(0)),
            reveal: Value::DevotionTo(vec![Color::Black]),
        })],
        ..creature(
            "Disciple of Phenax",
            cost(&[generic(2), b(), b()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Keepsake Gorgon — {3}{B}{B} 2/5 Gorgon with deathtouch. {5}{B}{B}:
/// Monstrosity 1; when it becomes monstrous, destroy target non-Gorgon creature
/// an opponent controls.
pub fn keepsake_gorgon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(5), b(), b()]), 1)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::Destroy {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByOpponent)
                    .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Gorgon)))),
            ),
        })],
        ..creature(
            "Keepsake Gorgon",
            cost(&[generic(3), b(), b()]),
            2,
            5,
            vec![CreatureType::Gorgon],
            vec![Keyword::Deathtouch],
        )
    }
}

/// Erebos's Emissary — {3}{B} 3/3 Snake. Bestow {5}{B}: +3/+3. Discard a
/// creature card: +2/+2 until end of turn — the enchanted creature instead
/// while this is a bestowed Aura.
pub fn erebos_s_emissary() -> CardDefinition {
    let pump = |what: Selector| Effect::PumpPT {
        what,
        power: Value::Const(2),
        toughness: Value::Const(2),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(cost(&[generic(5), b()])),
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 3, ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Creature, 1)),
            effect: Effect::If {
                cond: Predicate::SourceIsBestowedAura,
                then: Box::new(pump(Selector::AttachedTo(Box::new(Selector::This)))),
                else_: Box::new(pump(Selector::This)),
            },
            ..Default::default()
        }],
        ..creature(
            "Erebos's Emissary",
            cost(&[generic(3), b()]),
            3,
            3,
            vec![CreatureType::Snake],
            vec![],
        )
    }
}

/// Nighthowler — {1}{B}{B} 0/0 Horror. Bestow {2}{B}{B}. It and the enchanted
/// creature each get +X/+X, where X is the number of creature cards in all
/// graveyards.
pub fn nighthowler() -> CardDefinition {
    use crate::card::{DynamicPt, EquipScale};
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(cost(&[generic(2), b(), b()])),
        dynamic_pt: Some(DynamicPt::CreatureCardsInAllGraveyards { base_p: 0, base_t: 0 }),
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                per_power: 1,
                per_toughness: 1,
                count_all_graveyards: Some(R::Creature),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..creature("Nighthowler", cost(&[generic(1), b(), b()]), 0, 0, vec![CreatureType::Horror], vec![])
    }
}

/// Mogis's Marauder — {2}{B} 2/2 Human Berserker. ETB: up to X target creatures
/// gain intimidate and haste until end of turn, X = your devotion to black.
pub fn mogiss_marauder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CapTargetsAt {
            amount: Value::DevotionTo(vec![Color::Black]),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 5,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Intimidate,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
        })],
        ..creature(
            "Mogis's Marauder",
            cost(&[generic(2), b()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Berserker],
            vec![],
        )
    }
}

/// Scourgemark — {1}{B} Aura. Enchant creature. ETB: draw a card. Enchanted
/// creature gets +1/+0.
pub fn scourgemark() -> CardDefinition {
    CardDefinition {
        name: "Scourgemark",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        equipped_bonus: Some(EquipBonus { power: 1, ..Default::default() }),
        ..Default::default()
    }
}

/// Viper's Kiss — {B} Aura. Enchant creature. Enchanted creature gets -1/-1 and
/// its activated abilities can't be activated.
pub fn vipers_kiss() -> CardDefinition {
    CardDefinition {
        name: "Viper's Kiss",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: -1,
            toughness: -1,
            keywords: vec![Keyword::CantActivateAbilities],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Satyr Rambler — {1}{R} 2/1 Satyr with trample.
pub fn satyr_rambler() -> CardDefinition {
    creature(
        "Satyr Rambler",
        cost(&[generic(1), r()]),
        2,
        1,
        vec![CreatureType::Satyr],
        vec![Keyword::Trample],
    )
}

/// Firedrinker Satyr — {R} 2/1 Satyr Shaman. Whenever it's dealt damage, it
/// deals that much to you. {1}{R}: +1/+0 and 1 damage to you.
pub fn firedrinker_satyr() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::DealDamage { to: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Firedrinker Satyr",
            cost(&[r()]),
            2,
            1,
            vec![CreatureType::Satyr, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Arena Athlete — {1}{R} 2/1 Human. Heroic: target creature an opponent
/// controls can't block this turn.
pub fn arena_athlete() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Arena Athlete", cost(&[generic(1), r()]), 2, 1, vec![CreatureType::Human], vec![])
    }
}

/// Labyrinth Champion — {3}{R} 2/2 Human Warrior. Heroic: 2 damage to any target.
pub fn labyrinth_champion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::DealDamage {
            to: target_any(),
            amount: Value::Const(2),
        })],
        ..creature(
            "Labyrinth Champion",
            cost(&[generic(3), r()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Flamespeaker Adept — {2}{R} 2/3 Human Shaman. Whenever you scry, it gets
/// +2/+0 and gains first strike until end of turn.
pub fn flamespeaker_adept() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature(
            "Flamespeaker Adept",
            cost(&[generic(2), r()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Wild Celebrants — {3}{R}{R} 5/3 Satyr. ETB: you may destroy target artifact.
pub fn wild_celebrants() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            body: Box::new(Effect::Destroy { what: target_filtered(R::Artifact) }),
            description: String::new(),
        })],
        ..creature(
            "Wild Celebrants",
            cost(&[generic(3), r(), r()]),
            5,
            3,
            vec![CreatureType::Satyr],
            vec![],
        )
    }
}

/// Stoneshock Giant — {3}{R}{R} 5/4 Giant. {6}{R}{R}: Monstrosity 3; when it
/// becomes monstrous, creatures without flying your opponents control can't
/// block this turn.
pub fn stoneshock_giant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(6), r(), r()]), 3)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::GrantKeyword {
            what: Selector::EachPermanent(
                R::Creature.and(R::ControlledByOpponent).and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            ),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Stoneshock Giant",
            cost(&[generic(3), r(), r()]),
            5,
            4,
            vec![CreatureType::Giant],
            vec![],
        )
    }
}

/// Titan of Eternal Fire — {5}{R} 5/6 Giant. Each Human you control has
/// "{R}, {T}: this creature deals 1 damage to any target."
pub fn titan_of_eternal_fire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each Human creature you control has \"{R}, {T}: This creature \
                          deals 1 damage to any target.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasCreatureType(CreatureType::Human)),
                ),
                ability: ActivatedAbility {
                    mana_cost: cost(&[r()]),
                    tap_cost: true,
                    effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..creature(
            "Titan of Eternal Fire",
            cost(&[generic(5), r()]),
            5,
            6,
            vec![CreatureType::Giant],
            vec![],
        )
    }
}

/// Spark Jolt — {R} Instant. 1 damage to any target. Scry 1.
pub fn spark_jolt() -> CardDefinition {
    spell(
        "Spark Jolt",
        cost(&[r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::ONE },
            scry(1),
        ]),
    )
}

/// Rage of Purphoros — {4}{R} Sorcery. 4 damage to target creature; it can't be
/// regenerated this turn. Scry 1.
pub fn rage_of_purphoros() -> CardDefinition {
    spell(
        "Rage of Purphoros",
        cost(&[generic(4), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Creature) },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(4) },
            scry(1),
        ]),
    )
}

/// Peak Eruption — {2}{R} Sorcery. Destroy target Mountain; 3 damage to that
/// land's controller.
pub fn peak_eruption() -> CardDefinition {
    spell(
        "Peak Eruption",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::HasLandType(LandType::Mountain)) },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
        ]),
    )
}

/// Purphoros's Emissary — {3}{R} 3/3 Ox with menace. Bestow {6}{R}: +3/+3 and
/// menace.
pub fn purphoross_emissary() -> CardDefinition {
    bestow_creature(
        "Purphoros's Emissary",
        cost(&[generic(3), r()]),
        cost(&[generic(6), r()]),
        (3, 3),
        vec![CreatureType::Ox],
        vec![Keyword::Menace],
        (3, 3),
    )
}

/// Spearpoint Oread — {2}{R} 2/2 Nymph with first strike. Bestow {5}{R}: +2/+2
/// and first strike.
pub fn spearpoint_oread() -> CardDefinition {
    bestow_creature(
        "Spearpoint Oread",
        cost(&[generic(2), r()]),
        cost(&[generic(5), r()]),
        (2, 2),
        vec![CreatureType::Nymph],
        vec![Keyword::FirstStrike],
        (2, 2),
    )
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Satyr Hedonist — {1}{G} 2/1 Satyr. {R}, Sacrifice this: add {R}{R}{R}.
pub fn satyr_hedonist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::Const(3)),
            },
            ..Default::default()
        }],
        ..creature("Satyr Hedonist", cost(&[generic(1), g()]), 2, 1, vec![CreatureType::Satyr], vec![])
    }
}

/// Satyr Piper — {2}{G} 2/1 Satyr Rogue. {3}{G}: target creature must be
/// blocked this turn if able.
pub fn satyr_piper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::MustBeBlocked,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Satyr Piper",
            cost(&[generic(2), g()]),
            2,
            1,
            vec![CreatureType::Satyr, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Staunch-Hearted Warrior — {3}{G} 2/2 Human Warrior. Heroic: two +1/+1
/// counters.
pub fn staunch_hearted_warrior() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(counters_on_self(2))],
        ..creature(
            "Staunch-Hearted Warrior",
            cost(&[generic(3), g()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Centaur Battlemaster — {3}{G}{G} 3/3 Centaur Warrior. Heroic: three +1/+1
/// counters.
pub fn centaur_battlemaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(counters_on_self(3))],
        ..creature(
            "Centaur Battlemaster",
            cost(&[generic(3), g(), g()]),
            3,
            3,
            vec![CreatureType::Centaur, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Reverent Hunter — {2}{G} 1/1 Human Archer. ETB: +1/+1 counters equal to your
/// devotion to green.
pub fn reverent_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::DevotionTo(vec![Color::Green]),
        })],
        ..creature(
            "Reverent Hunter",
            cost(&[generic(2), g()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Archer],
            vec![],
        )
    }
}

/// Karametra's Acolyte — {3}{G} 1/4 Human Druid. {T}: add {G} equal to your
/// devotion to green.
pub fn karametras_acolyte() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::DevotionTo(vec![Color::Green])),
            },
            ..Default::default()
        }],
        ..creature(
            "Karametra's Acolyte",
            cost(&[generic(3), g()]),
            1,
            4,
            vec![CreatureType::Human, CreatureType::Druid],
            vec![],
        )
    }
}

/// Nemesis of Mortals — {4}{G}{G} 5/5 Snake. Costs {1} less per creature card in
/// your graveyard; {7}{G}{G}: Monstrosity 5, reduced the same way.
pub fn nemesis_of_mortals() -> CardDefinition {
    CardDefinition {
        affinity_graveyard_filter: Some(R::Creature),
        activated_abilities: vec![ActivatedAbility {
            cost_reduction_per_graveyard: Some(R::Creature),
            ..monstrosity(cost(&[generic(7), g(), g()]), 5)
        }],
        ..creature(
            "Nemesis of Mortals",
            cost(&[generic(4), g(), g()]),
            5,
            5,
            vec![CreatureType::Snake],
            vec![],
        )
    }
}

/// Shredding Winds — {2}{G} Instant. 7 damage to target creature with flying.
pub fn shredding_winds() -> CardDefinition {
    spell(
        "Shredding Winds",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            amount: Value::Const(7),
        },
    )
}

/// Artisan's Sorrow — {3}{G} Instant. Destroy target artifact or enchantment.
/// Scry 2.
pub fn artisans_sorrow() -> CardDefinition {
    spell(
        "Artisan's Sorrow",
        cost(&[generic(3), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            scry(2),
        ]),
    )
}

/// Boon Satyr — {1}{G}{G} 4/2 Satyr with flash. Bestow {3}{G}{G}: +4/+2.
pub fn boon_satyr() -> CardDefinition {
    bestow_creature(
        "Boon Satyr",
        cost(&[generic(1), g(), g()]),
        cost(&[generic(3), g(), g()]),
        (4, 2),
        vec![CreatureType::Satyr],
        vec![Keyword::Flash],
        (4, 2),
    )
}

// ── Multicolor / artifact ───────────────────────────────────────────────────

/// Akroan Hoplite — {R}{W} 1/2 Human Soldier. Whenever it attacks, it gets
/// +X/+0, where X is the number of attacking creatures you control.
pub fn akroan_hoplite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::CountOf(Box::new(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
            ))),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Akroan Hoplite",
            cost(&[r(), w()]),
            1,
            2,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Anax and Cymede — {1}{R}{W} 3/2 legendary Human Soldier with first strike and
/// vigilance. Heroic: creatures you control get +1/+1 and gain trample.
pub fn anax_and_cymede() -> CardDefinition {
    let team = Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![heroic(Effect::Seq(vec![
            Effect::PumpPT {
                what: team.clone(),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: team,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Anax and Cymede",
            cost(&[generic(1), r(), w()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![Keyword::FirstStrike, Keyword::Vigilance],
        )
    }
}

/// Chronicler of Heroes — {1}{G}{W} 3/3 Centaur Wizard. ETB: draw a card if you
/// control a creature with a +1/+1 counter on it.
pub fn chronicler_of_heroes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::WithCounter(CounterType::PlusOnePlusOne)),
                    ),
                    n: Value::ONE,
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Chronicler of Heroes",
            cost(&[generic(1), g(), w()]),
            3,
            3,
            vec![CreatureType::Centaur, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Kragma Warcaller — {3}{B}{R} 2/3 Minotaur Warrior. Minotaurs you control have
/// haste; whenever a Minotaur you control attacks, it gets +2/+0.
pub fn kragma_warcaller() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Minotaur creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasCreatureType(CreatureType::Minotaur)),
                ),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Minotaur),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Kragma Warcaller",
            cost(&[generic(3), b(), r()]),
            2,
            3,
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Pharika's Mender — {3}{B}{G} 4/3 Gorgon. ETB: you may return target creature
/// or enchantment card from your graveyard to your hand.
pub fn pharikas_mender() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            body: Box::new(Effect::Move {
                what: target_filtered(
                    R::InYourGraveyard.and(R::Creature.or(R::Enchantment)),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            description: String::new(),
        })],
        ..creature(
            "Pharika's Mender",
            cost(&[generic(3), b(), g()]),
            4,
            3,
            vec![CreatureType::Gorgon],
            vec![],
        )
    }
}

/// Horizon Chimera — {2}{G}{U} 3/2 Chimera with flash, flying and trample.
/// Whenever you draw a card, gain 1 life.
pub fn horizon_chimera() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Horizon Chimera",
            cost(&[generic(2), g(), u()]),
            3,
            2,
            vec![CreatureType::Chimera],
            vec![Keyword::Flash, Keyword::Flying, Keyword::Trample],
        )
    }
}

/// Destructive Revelry — {R}{G} Instant. Destroy target artifact or enchantment;
/// 2 damage to that permanent's controller.
pub fn destructive_revelry() -> CardDefinition {
    spell(
        "Destructive Revelry",
        cost(&[r(), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        ]),
    )
}

/// Cutthroat Maneuver — {3}{B} Instant. Up to two target creatures each get
/// +1/+1 and gain lifelink until end of turn.
pub fn cutthroat_maneuver() -> CardDefinition {
    spell(
        "Cutthroat Maneuver",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
    )
}

/// Witches' Eye — {1} Equipment. Equipped creature has "{1}, {T}: Scry 1."
/// Equip {1}.
pub fn witches_eye() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Witches' Eye",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: scry(1),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
