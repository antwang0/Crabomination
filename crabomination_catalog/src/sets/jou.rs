//! Journey into Nyx (JOU) — the primitive-free common/uncommon core plus the
//! monstrosity and heroic tails. Tests in `classic_sets/jou`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, heroic, monstrosity, on_becomes_monstrous, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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
        subtypes: Subtypes {
            creature_types: ct,
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
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

/// A plain "enchant creature" Aura.
fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// A JOU "Font" — an enchantment whose only text is one sacrifice ability.
fn font(name: &'static str, mana: ManaCost, sac_cost: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: sac_cost,
            sac_cost: true,
            effect,
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Oreskos Swiftclaw — {1}{W} 3/1 vanilla.
pub fn oreskos_swiftclaw() -> CardDefinition {
    creature(
        "Oreskos Swiftclaw",
        cost(&[generic(1), w()]),
        3,
        1,
        vec![CreatureType::Cat, CreatureType::Warrior],
        vec![],
    )
}

/// Pensive Minotaur — {2}{R} 2/3 vanilla.
pub fn pensive_minotaur() -> CardDefinition {
    creature(
        "Pensive Minotaur",
        cost(&[generic(2), r()]),
        2,
        3,
        vec![CreatureType::Minotaur, CreatureType::Warrior],
        vec![],
    )
}

/// Eagle of the Watch — {2}{W} 2/1 flying vigilance.
pub fn eagle_of_the_watch() -> CardDefinition {
    creature(
        "Eagle of the Watch",
        cost(&[generic(2), w()]),
        2,
        1,
        vec![CreatureType::Bird],
        vec![Keyword::Flying, Keyword::Vigilance],
    )
}

/// Bassara Tower Archer — {G}{G} 2/1 hexproof reach.
pub fn bassara_tower_archer() -> CardDefinition {
    creature(
        "Bassara Tower Archer",
        cost(&[g(), g()]),
        2,
        1,
        vec![CreatureType::Human, CreatureType::Archer],
        vec![Keyword::Hexproof, Keyword::Reach],
    )
}

/// Cloaked Siren — {3}{U} 3/2 flash flier.
pub fn cloaked_siren() -> CardDefinition {
    creature(
        "Cloaked Siren",
        cost(&[generic(3), u()]),
        3,
        2,
        vec![CreatureType::Siren],
        vec![Keyword::Flash, Keyword::Flying],
    )
}

/// Gold-Forged Sentinel — {6} 4/4 flying artifact creature.
pub fn gold_forged_sentinel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(
            "Gold-Forged Sentinel",
            cost(&[generic(6)]),
            4,
            4,
            vec![CreatureType::Chimera],
            vec![Keyword::Flying],
        )
    }
}

/// Golden Hind — {1}{G} 2/1 that taps for {G}.
pub fn golden_hind() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Golden Hind",
            cost(&[generic(1), g()]),
            2,
            1,
            vec![CreatureType::Elk],
            vec![],
        )
    }
}

/// Akroan Mastiff — {3}{W} 2/2. {W}, {T}: Tap target creature.
pub fn akroan_mastiff() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Akroan Mastiff",
            cost(&[generic(3), w()]),
            2,
            2,
            vec![CreatureType::Dog],
            vec![],
        )
    }
}

/// Akroan Line Breaker — {2}{R} 2/1. Heroic: +2/+0 and intimidate.
pub fn akroan_line_breaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(crate::effect::shortcut::pump_and_grant_keyword(
            2,
            0,
            Keyword::Intimidate,
        ))],
        ..creature(
            "Akroan Line Breaker",
            cost(&[generic(2), r()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Lagonna-Band Trailblazer — {W} 0/4. Heroic: a +1/+1 counter.
pub fn lagonna_band_trailblazer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Lagonna-Band Trailblazer",
            cost(&[w()]),
            0,
            4,
            vec![CreatureType::Centaur, CreatureType::Scout],
            vec![],
        )
    }
}

/// Pheres-Band Thunderhoof — {4}{G} 3/4. Heroic: two +1/+1 counters.
pub fn pheres_band_thunderhoof() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..creature(
            "Pheres-Band Thunderhoof",
            cost(&[generic(4), g()]),
            3,
            4,
            vec![CreatureType::Centaur, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Dawnbringer Charioteers — {2}{W}{W} 2/4 flying lifelink. Heroic: a counter.
pub fn dawnbringer_charioteers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Dawnbringer Charioteers",
            cost(&[generic(2), w(), w()]),
            2,
            4,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![Keyword::Flying, Keyword::Lifelink],
        )
    }
}

/// Leonin Iconoclast — {3}{W} 3/2. Heroic: destroy an opposing enchantment
/// creature.
pub fn leonin_iconoclast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Destroy {
            what: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.and(R::Enchantment).and(R::ControlledByOpponent),
            },
        })],
        ..creature(
            "Leonin Iconoclast",
            cost(&[generic(3), w()]),
            3,
            2,
            vec![CreatureType::Cat, CreatureType::Monk],
            vec![],
        )
    }
}

/// Felhide Petrifier — {2}{B} 2/3 deathtouch; your other Minotaurs have it too.
pub fn felhide_petrifier() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Minotaur creatures you control have deathtouch.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Minotaur))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                keyword: Keyword::Deathtouch,
            },
        }],
        ..creature(
            "Felhide Petrifier",
            cost(&[generic(2), b()]),
            2,
            3,
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            vec![Keyword::Deathtouch],
        )
    }
}

/// Pheres-Band Warchief — {3}{G} 3/3 vigilance trample; a Centaur lord.
pub fn pheres_band_warchief() -> CardDefinition {
    let others = || {
        Selector::EachPermanent(
            R::Creature
                .and(R::HasCreatureType(CreatureType::Centaur))
                .and(R::ControlledByYou)
                .and(R::OtherThanSource),
        )
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other Centaur creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: others(),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Other Centaur creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: others(),
                    keyword: Keyword::Vigilance,
                },
            },
            StaticAbility {
                description: "Other Centaur creatures you control have trample.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: others(),
                    keyword: Keyword::Trample,
                },
            },
        ],
        ..creature(
            "Pheres-Band Warchief",
            cost(&[generic(3), g()]),
            3,
            3,
            vec![CreatureType::Centaur, CreatureType::Warrior],
            vec![Keyword::Vigilance, Keyword::Trample],
        )
    }
}

/// Cyclops of Eternal Fury — {4}{R}{R} 5/3 enchantment creature; your creatures
/// have haste.
pub fn cyclops_of_eternal_fury() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Haste,
            },
        }],
        ..creature(
            "Cyclops of Eternal Fury",
            cost(&[generic(4), r(), r()]),
            5,
            3,
            vec![CreatureType::Cyclops],
            vec![],
        )
    }
}

/// Gluttonous Cyclops — {5}{R} 5/4 with monstrosity 3.
pub fn gluttonous_cyclops() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(5), r(), r()]), 3)],
        ..creature(
            "Gluttonous Cyclops",
            cost(&[generic(5), r()]),
            5,
            4,
            vec![CreatureType::Cyclops],
            vec![],
        )
    }
}

/// Fleetfeather Cockatrice — {3}{G}{U} 3/3 flash flier with deathtouch and
/// monstrosity 3.
pub fn fleetfeather_cockatrice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(5), g(), u()]), 3)],
        ..creature(
            "Fleetfeather Cockatrice",
            cost(&[generic(3), g(), u()]),
            3,
            3,
            vec![CreatureType::Cockatrice],
            vec![Keyword::Flash, Keyword::Flying, Keyword::Deathtouch],
        )
    }
}

/// Hydra Broodmaster — {4}{G}{G} 7/7. Monstrosity X mints X X/X Hydras.
pub fn hydra_broodmaster() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), x(), g()]),
            effect: Effect::Monstrosity {
                n: Value::XFromCost,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![on_becomes_monstrous(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::TriggerEventAmount,
            definition: TokenDefinition {
                name: "Hydra".into(),
                colors: vec![Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Hydra],
                    ..Default::default()
                },
                dynamic_pt: Some((Value::TriggerEventAmount, Value::TriggerEventAmount)),
                ..Default::default()
            },
        })],
        ..creature(
            "Hydra Broodmaster",
            cost(&[generic(4), g(), g()]),
            7,
            7,
            vec![CreatureType::Hydra],
            vec![],
        )
    }
}

/// King Macar, the Gold-Cursed — {2}{B}{B} 2/3. Inspired: exile a creature for
/// a Gold token.
pub fn king_macar_the_gold_cursed() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: crate::card::EventSpec::new(
                crate::card::EventKind::BecomesUntapped,
                crate::card::EventScope::SelfSource,
            ),
            effect: Effect::MayDo {
                description: "Exile target creature for a Gold token".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile {
                        what: target_filtered(R::Creature),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: super::thb::gold_token(),
                    },
                ])),
            },
        }],
        ..creature(
            "King Macar, the Gold-Cursed",
            cost(&[generic(2), b(), b()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Noble],
            vec![],
        )
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Countermand — {2}{U}{U} Instant. Counter a spell; its controller mills four.
pub fn countermand() -> CardDefinition {
    spell(
        "Countermand",
        cost(&[generic(2), u(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack),
            },
            Effect::Mill {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(4),
            },
        ]),
    )
}

/// Desecration Plague — {3}{G} Sorcery. Destroy target enchantment or land.
pub fn desecration_plague() -> CardDefinition {
    spell(
        "Desecration Plague",
        cost(&[generic(3), g()]),
        CardType::Sorcery,
        Effect::Destroy {
            what: target_filtered(R::Enchantment.or(R::Land)),
        },
    )
}

/// Extinguish All Hope — {4}{B}{B} Sorcery. Destroy all nonenchantment
/// creatures.
pub fn extinguish_all_hope() -> CardDefinition {
    spell(
        "Extinguish All Hope",
        cost(&[generic(4), b(), b()]),
        CardType::Sorcery,
        Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::Enchantment)))),
        },
    )
}

/// Feast of Dreams — {1}{B} Instant. Destroy an enchanted or enchantment
/// creature.
pub fn feast_of_dreams() -> CardDefinition {
    spell(
        "Feast of Dreams",
        cost(&[generic(1), b()]),
        CardType::Instant,
        Effect::Destroy {
            what: target_filtered(R::Creature.and(R::IsEnchanted.or(R::Enchantment))),
        },
    )
}

/// Flurry of Horns — {4}{R} Sorcery. Two hasty 2/3 Minotaurs.
pub fn flurry_of_horns() -> CardDefinition {
    spell(
        "Flurry of Horns",
        cost(&[generic(4), r()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: TokenDefinition {
                name: "Minotaur".into(),
                power: 2,
                toughness: 3,
                colors: vec![Color::Red],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Minotaur],
                    ..Default::default()
                },
                keywords: vec![Keyword::Haste],
                ..Default::default()
            },
        },
    )
}

/// Hubris — {1}{U} Instant. Bounce a creature and everything attached to it.
pub fn hubris() -> CardDefinition {
    spell(
        "Hubris",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::AttachedToMe(Box::new(target_filtered(R::Creature))),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
    )
}

/// Nightmarish End — {2}{B} Instant. -X/-X where X is your hand size.
pub fn nightmarish_end() -> CardDefinition {
    spell(
        "Nightmarish End",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Times(
                Box::new(Value::Const(-1)),
                Box::new(Value::CardsInHandMatching {
                    who: PlayerRef::You,
                    filter: R::Any,
                }),
            ),
            toughness: Value::Times(
                Box::new(Value::Const(-1)),
                Box::new(Value::CardsInHandMatching {
                    who: PlayerRef::You,
                    filter: R::Any,
                }),
            ),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Interpret the Signs — {5}{U} Sorcery. Scry 3, then draw the top card's mana
/// value.
pub fn interpret_the_signs() -> CardDefinition {
    spell(
        "Interpret the Signs",
        cost(&[generic(5), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(3),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::ONE,
                })),
            },
        ]),
    )
}

// ── Auras, Equipment, and the Fonts ──────────────────────────────────────────

/// Aspect of Gorgon — {2}{B} Aura. +1/+3 and deathtouch.
pub fn aspect_of_gorgon() -> CardDefinition {
    aura(
        "Aspect of Gorgon",
        cost(&[generic(2), b()]),
        EquipBonus {
            power: 1,
            toughness: 3,
            keywords: vec![Keyword::Deathtouch],
            ..Default::default()
        },
    )
}

/// Cast into Darkness — {1}{B} Aura. -2/-0 and can't block.
pub fn cast_into_darkness() -> CardDefinition {
    aura(
        "Cast into Darkness",
        cost(&[generic(1), b()]),
        EquipBonus {
            power: -2,
            keywords: vec![Keyword::CantBlock],
            ..Default::default()
        },
    )
}

/// Pin to the Earth — {1}{U} Aura. -6/-0.
pub fn pin_to_the_earth() -> CardDefinition {
    aura(
        "Pin to the Earth",
        cost(&[generic(1), u()]),
        EquipBonus {
            power: -6,
            ..Default::default()
        },
    )
}

/// Lightning Diadem — {5}{R} Aura. ETB 2 damage; +2/+2.
pub fn lightning_diadem() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
            },
            amount: Value::Const(2),
        })],
        ..aura(
            "Lightning Diadem",
            cost(&[generic(5), r()]),
            EquipBonus {
                power: 2,
                toughness: 2,
                ..Default::default()
            },
        )
    }
}

/// Nyx Infusion — {2}{B} Aura. +2/+2 on an enchantment, -2/-2 otherwise.
pub fn nyx_infusion() -> CardDefinition {
    aura(
        "Nyx Infusion",
        cost(&[generic(2), b()]),
        EquipBonus {
            power: -2,
            toughness: -2,
            conditional: vec![crate::card::ConditionalEquipBonus {
                host_filter: R::Enchantment,
                power: 4,
                toughness: 4,
                keywords: vec![],
                condition: None,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Mogis's Warhound — {1}{R} 2/2 that must attack; bestow {2}{R} passes the
/// compulsion along.
pub fn mogiss_warhound() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(cost(&[generic(2), r()])),
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::MustAttack],
            ..Default::default()
        }),
        ..creature(
            "Mogis's Warhound",
            cost(&[generic(1), r()]),
            2,
            2,
            vec![CreatureType::Dog],
            vec![Keyword::MustAttack],
        )
    }
}

/// Chariot of Victory — {3} Equipment. First strike, trample, haste. Equip {1}.
pub fn chariot_of_victory() -> CardDefinition {
    CardDefinition {
        name: "Chariot of Victory",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::FirstStrike, Keyword::Trample, Keyword::Haste],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Hall of Triumph — {3} Legendary Artifact. Choose a color; your creatures of
/// that color get +1/+1.
pub fn hall_of_triumph() -> CardDefinition {
    CardDefinition {
        name: "Hall of Triumph",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen color get +1/+1.",
            effect: StaticEffect::AnthemForChosenColor {
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Font of Fortunes — {1}{U}. {1}{U}, Sacrifice: draw two.
pub fn font_of_fortunes() -> CardDefinition {
    font(
        "Font of Fortunes",
        cost(&[generic(1), u()]),
        cost(&[generic(1), u()]),
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
    )
}

/// Font of Ire — {1}{R}. {3}{R}, Sacrifice: 5 damage to a player or walker.
pub fn font_of_ire() -> CardDefinition {
    font(
        "Font of Ire",
        cost(&[generic(1), r()]),
        cost(&[generic(3), r()]),
        Effect::DealDamage {
            to: target_filtered(R::Player.or(R::Planeswalker)),
            amount: Value::Const(5),
        },
    )
}

/// Font of Return — {1}{B}. {3}{B}, Sacrifice: return up to three creature
/// cards from your graveyard to hand.
pub fn font_of_return() -> CardDefinition {
    font(
        "Font of Return",
        cost(&[generic(1), b()]),
        cost(&[generic(3), b()]),
        Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature.and(R::InGraveyard),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    )
}

/// Font of Vigor — {1}{W}. {2}{W}, Sacrifice: gain 7 life.
pub fn font_of_vigor() -> CardDefinition {
    font(
        "Font of Vigor",
        cost(&[generic(1), w()]),
        cost(&[generic(2), w()]),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(7),
        },
    )
}
