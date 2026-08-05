//! Born of the Gods (BNG) — the rest of the set. Tests in `classic_sets/bng`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, heroic, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest,
};
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

/// "Inspired — Whenever this creature becomes untapped, …" (CR 702.108).
fn inspired(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
        effect,
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

/// A BNG "enchantment creature" token (the inspired cycle's payoffs).
fn nyx_token(
    name: &str,
    p: i32,
    t: i32,
    color: Color,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: ct,
            ..Default::default()
        },
        keywords: kw,
        ..Default::default()
    }
}

/// "Inspired — … you may pay `mana`. If you do, create `count` `token`s."
fn inspired_pay_for_tokens(mana: ManaCost, token: TokenDefinition, count: i32) -> TriggeredAbility {
    inspired(Effect::MayPay {
        description: format!("Pay for {}", token.name),
        mana_cost: mana,
        body: Box::new(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(count),
            definition: token,
        }),
        else_: None,
    })
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Swordwise Centaur — {G}{G} 3/2 vanilla.
pub fn swordwise_centaur() -> CardDefinition {
    creature(
        "Swordwise Centaur",
        cost(&[g(), g()]),
        3,
        2,
        vec![CreatureType::Centaur, CreatureType::Warrior],
        vec![],
    )
}

/// Oreskos Sun Guide — {1}{W} 2/2. Inspired: gain 2 life.
pub fn oreskos_sun_guide() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..creature(
            "Oreskos Sun Guide",
            cost(&[generic(1), w()]),
            2,
            2,
            vec![CreatureType::Cat, CreatureType::Monk],
            vec![],
        )
    }
}

/// Sphinx's Disciple — {3}{U}{U} 2/2 flier. Inspired: draw a card.
pub fn sphinxs_disciple() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Sphinx's Disciple",
            cost(&[generic(3), u(), u()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![Keyword::Flying],
        )
    }
}

/// Setessan Oathsworn — {1}{G}{G} 1/1. Heroic: two +1/+1 counters.
pub fn setessan_oathsworn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..creature(
            "Setessan Oathsworn",
            cost(&[generic(1), g(), g()]),
            1,
            1,
            vec![CreatureType::Satyr, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Vanguard of Brimaz — {W}{W} 2/2 vigilance. Heroic: a 1/1 Cat Soldier with
/// vigilance.
pub fn vanguard_of_brimaz() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Cat Soldier".into(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
                    ..Default::default()
                },
                keywords: vec![Keyword::Vigilance],
                ..Default::default()
            },
        })],
        ..creature(
            "Vanguard of Brimaz",
            cost(&[w(), w()]),
            2,
            2,
            vec![CreatureType::Cat, CreatureType::Soldier],
            vec![Keyword::Vigilance],
        )
    }
}

/// Setessan Starbreaker — {3}{G} 2/1. ETB: you may destroy target Aura.
pub fn setessan_starbreaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Destroy target Aura".into(),
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
            }),
        })],
        ..creature(
            "Setessan Starbreaker",
            cost(&[generic(3), g()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Stormcaller of Keranos — {2}{R} 2/2 haste. {1}{U}: Scry 1.
pub fn stormcaller_of_keranos() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Stormcaller of Keranos",
            cost(&[generic(2), r()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![Keyword::Haste],
        )
    }
}

/// Warchanter of Mogis — {3}{B}{B} 3/3. Inspired: a creature you control gains
/// intimidate.
pub fn warchanter_of_mogis() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Intimidate,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Warchanter of Mogis",
            cost(&[generic(3), b(), b()]),
            3,
            3,
            vec![CreatureType::Minotaur, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Siren of the Silent Song — {1}{U}{B} 2/1 flier. Inspired: each opponent
/// discards a card, then each opponent mills a card.
pub fn siren_of_the_silent_song() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            },
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        ]))],
        ..creature(
            "Siren of the Silent Song",
            cost(&[generic(1), u(), b()]),
            2,
            1,
            vec![CreatureType::Zombie, CreatureType::Siren],
            vec![Keyword::Flying],
        )
    }
}

/// Kiora's Follower — {G}{U} 2/2. {T}: Untap another target permanent.
pub fn kioras_follower() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap {
                what: target_filtered(R::Permanent.and(R::OtherThanSource)),
                up_to: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Kiora's Follower",
            cost(&[g(), u()]),
            2,
            2,
            vec![CreatureType::Merfolk],
            vec![],
        )
    }
}

/// Black Oak of Odunos — {2}{B} 0/5 defender. {B}, Tap another untapped
/// creature you control: +1/+1 until end of turn.
pub fn black_oak_of_odunos() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou).and(R::Untapped)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Black Oak of Odunos",
            cost(&[generic(2), b()]),
            0,
            5,
            vec![CreatureType::Zombie, CreatureType::Treefolk],
            vec![Keyword::Defender],
        )
    }
}

/// Reckless Reveler — {1}{R} 2/1. {R}, Sacrifice this: Destroy target artifact.
pub fn reckless_reveler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            ..Default::default()
        }],
        ..creature(
            "Reckless Reveler",
            cost(&[generic(1), r()]),
            2,
            1,
            vec![CreatureType::Satyr],
            vec![],
        )
    }
}

/// Pheres-Band Raiders — {5}{G} 5/5. Inspired: pay {2}{G} for a 3/3 Centaur
/// enchantment creature token.
pub fn pheres_band_raiders() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired_pay_for_tokens(
            cost(&[generic(2), g()]),
            nyx_token(
                "Centaur",
                3,
                3,
                Color::Green,
                vec![CreatureType::Centaur],
                vec![],
            ),
            1,
        )],
        ..creature(
            "Pheres-Band Raiders",
            cost(&[generic(5), g()]),
            5,
            5,
            vec![CreatureType::Centaur, CreatureType::Warrior],
            vec![],
        )
    }
}

/// God-Favored General — {1}{W} 1/1. Inspired: pay {2}{W} for two 1/1 Soldier
/// enchantment creature tokens.
pub fn god_favored_general() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired_pay_for_tokens(
            cost(&[generic(2), w()]),
            nyx_token(
                "Soldier",
                1,
                1,
                Color::White,
                vec![CreatureType::Soldier],
                vec![],
            ),
            2,
        )],
        ..creature(
            "God-Favored General",
            cost(&[generic(1), w()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Forlorn Pseudamma — {3}{B} 2/1 intimidate. Inspired: pay {2}{B} for a 2/2
/// Zombie enchantment creature token.
pub fn forlorn_pseudamma() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired_pay_for_tokens(
            cost(&[generic(2), b()]),
            nyx_token(
                "Zombie",
                2,
                2,
                Color::Black,
                vec![CreatureType::Zombie],
                vec![],
            ),
            1,
        )],
        ..creature(
            "Forlorn Pseudamma",
            cost(&[generic(3), b()]),
            2,
            1,
            vec![CreatureType::Zombie],
            vec![Keyword::Intimidate],
        )
    }
}

/// Satyr Nyx-Smith — {2}{R} 2/1 haste. Inspired: pay {2}{R} for a 3/1
/// Elemental enchantment creature token with haste.
pub fn satyr_nyx_smith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired_pay_for_tokens(
            cost(&[generic(2), r()]),
            nyx_token(
                "Elemental",
                3,
                1,
                Color::Red,
                vec![CreatureType::Elemental],
                vec![Keyword::Haste],
            ),
            1,
        )],
        ..creature(
            "Satyr Nyx-Smith",
            cost(&[generic(2), r()]),
            2,
            1,
            vec![CreatureType::Satyr, CreatureType::Shaman],
            vec![Keyword::Haste],
        )
    }
}

/// Aerie Worshippers — {3}{U} 2/4. Inspired: pay {2}{U} for a 2/2 flying Bird
/// enchantment creature token.
pub fn aerie_worshippers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired_pay_for_tokens(
            cost(&[generic(2), u()]),
            nyx_token(
                "Bird",
                2,
                2,
                Color::Blue,
                vec![CreatureType::Bird],
                vec![Keyword::Flying],
            ),
            1,
        )],
        ..creature(
            "Aerie Worshippers",
            cost(&[generic(3), u()]),
            2,
            4,
            vec![CreatureType::Human, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Kraken of the Straits — {5}{U}{U} 6/6. Creatures with power less than your
/// Island count can't block it.
pub fn kraken_of_the_straits() -> CardDefinition {
    creature(
        "Kraken of the Straits",
        cost(&[generic(5), u(), u()]),
        6,
        6,
        vec![CreatureType::Kraken],
        vec![Keyword::CantBeBlockedByPowerLessThanCount(Box::new(
            R::HasLandType(crate::card::LandType::Island),
        ))],
    )
}

/// Tromokratis — {5}{U}{U} 8/8 legend. Hexproof unless attacking or blocking;
/// can't be blocked unless every creature the defender controls blocks it.
pub fn tromokratis() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(
            "Tromokratis",
            cost(&[generic(5), u(), u()]),
            8,
            8,
            vec![CreatureType::Kraken],
            vec![
                Keyword::HexproofUnlessAttackingOrBlocking,
                Keyword::CantBeBlockedUnlessAllBlock,
            ],
        )
    }
}

/// Fate Unraveler — {3}{B} 3/4. Whenever an opponent draws a card, deal 1
/// damage to that player.
pub fn fate_unraveler() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Fate Unraveler",
            cost(&[generic(3), b()]),
            3,
            4,
            vec![CreatureType::Hag],
            vec![],
        )
    }
}

/// Pain Seer — {1}{B} 2/2. Inspired: reveal the top card, put it into your
/// hand, and lose life equal to its mana value.
pub fn pain_seer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::Seq(vec![
            // Read the mana value off the live top card before it moves.
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::ONE,
                })),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]))],
        ..creature(
            "Pain Seer",
            cost(&[generic(1), b()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Odunos River Trawler — {2}{B} 2/2. ETB and a {W}-sacrifice both return an
/// enchantment creature card from your graveyard to hand.
pub fn odunos_river_trawler() -> CardDefinition {
    let recur = || Effect::Move {
        what: target_filtered(R::Creature.and(R::Enchantment).and(R::InGraveyard)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        triggered_abilities: vec![etb(recur())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: recur(),
            ..Default::default()
        }],
        ..creature(
            "Odunos River Trawler",
            cost(&[generic(2), b()]),
            2,
            2,
            vec![CreatureType::Zombie],
            vec![],
        )
    }
}

/// Scourge of Skola Vale — {2}{G} 0/0 trample; enters with two +1/+1 counters.
/// {T}, Sacrifice another creature: counters equal to its toughness.
pub fn scourge_of_skola_vale() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::SacrificedToughness,
            },
            ..Default::default()
        }],
        ..creature(
            "Scourge of Skola Vale",
            cost(&[generic(2), g()]),
            0,
            0,
            vec![CreatureType::Hydra],
            vec![Keyword::Trample],
        )
    }
}

/// Eater of Hope — {5}{B}{B} 6/4 flier. Sac a creature to regenerate; sac two
/// to destroy target creature.
pub fn eater_of_hope() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Regenerate {
                    what: Selector::This,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                sac_other_filter: Some((R::Creature, 2)),
                effect: Effect::Destroy {
                    what: target_filtered(R::Creature),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Eater of Hope",
            cost(&[generic(5), b(), b()]),
            6,
            4,
            vec![CreatureType::Demon],
            vec![Keyword::Flying],
        )
    }
}

/// Forgestoker Dragon — {4}{R}{R} 5/4 flier. While attacking, {1}{R}: 1 damage
/// to target creature; it can't block this combat.
pub fn forgestoker_dragon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            condition: Some(crate::effect::Predicate::EntityMatches {
                what: Selector::This,
                filter: R::IsAttacking,
            }),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Forgestoker Dragon",
            cost(&[generic(4), r(), r()]),
            5,
            4,
            vec![CreatureType::Dragon],
            vec![Keyword::Flying],
        )
    }
}

/// Silent Sentinel — {5}{W}{W} 4/6 flier. Attacks: return target enchantment
/// card from your graveyard to the battlefield.
pub fn silent_sentinel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::MayDo {
            description: "Return an enchantment from your graveyard".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Enchantment.and(R::InGraveyard)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            }),
        })],
        ..creature(
            "Silent Sentinel",
            cost(&[generic(5), w(), w()]),
            4,
            6,
            vec![CreatureType::Archon],
            vec![Keyword::Flying],
        )
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Rise to the Challenge — {1}{R} Instant. +2/+0 and first strike.
pub fn rise_to_the_challenge() -> CardDefinition {
    spell(
        "Rise to the Challenge",
        cost(&[generic(1), r()]),
        CardType::Instant,
        crate::effect::shortcut::pump_and_grant_keyword(2, 0, Keyword::FirstStrike),
    )
}

/// Mischief and Mayhem — {4}{G} Sorcery. Up to two target creatures get +4/+4.
pub fn mischief_and_mayhem() -> CardDefinition {
    spell(
        "Mischief and Mayhem",
        cost(&[generic(4), g()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Reap What Is Sown — {1}{G}{W} Instant. A +1/+1 counter on each of up to
/// three target creatures.
pub fn reap_what_is_sown() -> CardDefinition {
    spell(
        "Reap What Is Sown",
        cost(&[generic(1), g(), w()]),
        CardType::Instant,
        Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 1,
            filter: R::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        },
    )
}

/// Pinnacle of Rage — {4}{R}{R} Sorcery. 3 damage to each of two targets.
pub fn pinnacle_of_rage() -> CardDefinition {
    spell(
        "Pinnacle of Rage",
        cost(&[generic(4), r(), r()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature.or(R::Player).or(R::Planeswalker),
            effect: Box::new(Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(3),
            }),
        },
    )
}

/// Scouring Sands — {1}{R} Sorcery. 1 damage to each creature your opponents
/// control, then scry 1.
pub fn scouring_sands() -> CardDefinition {
    spell(
        "Scouring Sands",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::ONE,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
    )
}

/// Unravel the Aether — {1}{G} Instant. Shuffle target artifact or enchantment
/// into its owner's library.
pub fn unravel_the_aether() -> CardDefinition {
    spell(
        "Unravel the Aether",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Move {
            what: target_filtered(R::Artifact.or(R::Enchantment)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: LibraryPosition::Shuffled,
            },
        },
    )
}

/// Gild — {3}{B} Sorcery. Exile target creature; create a Gold token.
pub fn gild() -> CardDefinition {
    spell(
        "Gild",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::Creature),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: super::thb::gold_token(),
            },
        ]),
    )
}

/// Dawn to Dusk — {2}{W}{W} Sorcery. Choose one or both: return an enchantment
/// card from your graveyard to hand; destroy target enchantment.
pub fn dawn_to_dusk() -> CardDefinition {
    spell(
        "Dawn to Dusk",
        cost(&[generic(2), w(), w()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Move {
                    what: target_filtered(R::Enchantment.and(R::InGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Destroy {
                    what: target_filtered(R::Enchantment),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Whelming Wave — {2}{U}{U} Sorcery. Bounce every creature except Krakens,
/// Leviathans, Octopuses, and Serpents.
pub fn whelming_wave() -> CardDefinition {
    let spared = R::HasCreatureType(CreatureType::Kraken)
        .or(R::HasCreatureType(CreatureType::Leviathan))
        .or(R::HasCreatureType(CreatureType::Octopus))
        .or(R::HasCreatureType(CreatureType::Serpent));
    spell(
        "Whelming Wave",
        cost(&[generic(2), u(), u()]),
        CardType::Sorcery,
        Effect::Move {
            what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(spared)))),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Sudden Storm — {3}{U} Instant. Tap up to two target creatures; they don't
/// untap next untap step. Scry 1.
pub fn sudden_storm() -> CardDefinition {
    spell(
        "Sudden Storm",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::TapUpToValue {
                count: Value::Const(2),
                filter: R::Creature,
                skip_untap: true,
                exact: false,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
    )
}

/// Peregrination — {3}{G} Sorcery. Fetch two basics — one onto the battlefield
/// tapped, one to hand — then scry 1.
pub fn peregrination() -> CardDefinition {
    spell(
        "Peregrination",
        cost(&[generic(3), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
    )
}

/// Sanguimancy — {4}{B} Sorcery. Draw X and lose X, X = devotion to black.
pub fn sanguimancy() -> CardDefinition {
    spell(
        "Sanguimancy",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::DevotionTo(vec![Color::Black]),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::DevotionTo(vec![Color::Black]),
            },
        ]),
    )
}

/// Skyreaping — {1}{G} Sorcery. Damage to each flier equal to your devotion to
/// green.
pub fn skyreaping() -> CardDefinition {
    spell(
        "Skyreaping",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            amount: Value::DevotionTo(vec![Color::Green]),
        },
    )
}

/// Thassa's Rebuff — {1}{U} Instant. Counter target spell unless its controller
/// pays {X}, X = your devotion to blue.
pub fn thassas_rebuff() -> CardDefinition {
    spell(
        "Thassa's Rebuff",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: ManaCost::default(),
            exile: false,
            extra_generic: Some(Value::DevotionTo(vec![Color::Blue])),
        },
    )
}

/// Glimpse-style tap cycle sibling — Lightning Volley, {3}{R} Instant. Until
/// end of turn, your creatures gain "{T}: 1 damage to any target."
pub fn lightning_volley() -> CardDefinition {
    spell(
        "Lightning Volley",
        cost(&[generic(3), r()]),
        CardType::Instant,
        Effect::GainActivatedAbility {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            ability: Box::new(ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: target_any(),
                    amount: Value::ONE,
                },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Retraction Helix — {U} Instant. Target creature gains "{T}: Return target
/// nonland permanent to its owner's hand" until end of turn.
pub fn retraction_helix() -> CardDefinition {
    spell(
        "Retraction Helix",
        cost(&[u()]),
        CardType::Instant,
        Effect::GainActivatedAbility {
            what: target_filtered(R::Creature),
            ability: Box::new(ActivatedAbility {
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Permanent.and(R::Not(Box::new(R::Land)))),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Hunter's Prowess — {4}{G} Sorcery. +3/+3, trample, and "deals combat damage
/// to a player: draw that many cards" until end of turn.
pub fn hunters_prowess() -> CardDefinition {
    spell(
        "Hunter's Prowess",
        cost(&[generic(4), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToPlayer,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::Draw {
                        who: Selector::You,
                        amount: Value::TriggerEventAmount,
                    },
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

// ── Auras and Equipment ──────────────────────────────────────────────────────

/// Weight of the Underworld — {3}{B} Aura. Enchanted creature gets -3/-2.
pub fn weight_of_the_underworld() -> CardDefinition {
    aura(
        "Weight of the Underworld",
        cost(&[generic(3), b()]),
        EquipBonus {
            power: -3,
            toughness: -2,
            ..Default::default()
        },
    )
}

/// Oracle's Insight — {3}{U} Aura. Enchanted creature has "{T}: Scry 1, then
/// draw a card."
pub fn oracles_insight() -> CardDefinition {
    aura(
        "Oracle's Insight",
        cost(&[generic(3), u()]),
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::ONE,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Stratus Walk — {1}{U} Aura. ETB draw; enchanted creature has flying and can
/// block only creatures with flying.
pub fn stratus_walk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..aura(
            "Stratus Walk",
            cost(&[generic(1), u()]),
            EquipBonus {
                keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
                ..Default::default()
            },
        )
    }
}

/// Raised by Wolves — {3}{G}{G} Aura. ETB: two 2/2 Wolves. Enchanted creature
/// gets +1/+1 for each Wolf you control.
pub fn raised_by_wolves() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: TokenDefinition {
                name: "Wolf".into(),
                power: 2,
                toughness: 2,
                colors: vec![Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Wolf],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..aura(
            "Raised by Wolves",
            cost(&[generic(3), g(), g()]),
            EquipBonus {
                scale: Some(EquipScale {
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Wolf)),
                    per_power: 1,
                    per_toughness: 1,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}

/// Ephara's Enlightenment — {1}{W}{U} Aura. ETB counter; enchanted creature has
/// flying; your creature ETBs may return this Aura to hand.
pub fn epharas_enlightenment() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(crate::effect::Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: Effect::MayDo {
                    description: "Return Ephara's Enlightenment to your hand".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                },
            },
        ],
        ..aura(
            "Ephara's Enlightenment",
            cost(&[generic(1), w(), u()]),
            EquipBonus {
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
        )
    }
}

/// Thunderous Might — {1}{R} Aura. Enchanted creature attacks: +X/+0, X = your
/// devotion to red.
pub fn thunderous_might() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
            effect: Effect::PumpPT {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::DevotionTo(vec![Color::Red]),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..aura(
            "Thunderous Might",
            cost(&[generic(1), r()]),
            EquipBonus::default(),
        )
    }
}

/// Siren Song Lyre — {2} Equipment. Equipped creature has "{2}, {T}: Tap target
/// creature." Equip {2}.
pub fn siren_song_lyre() -> CardDefinition {
    CardDefinition {
        name: "Siren Song Lyre",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Tap {
                    what: target_filtered(R::Creature),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Astral Cornucopia — {X}{X}{X} Artifact. Enters with X charge counters;
/// {T}: add one mana of a chosen color per charge counter.
pub fn astral_cornucopia() -> CardDefinition {
    CardDefinition {
        name: "Astral Cornucopia",
        cost: cost(&[x(), x(), x()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Heroes' Podium — {5} Legendary Artifact. Legendary lord; {X}, {T}: dig X for
/// a legendary creature card.
pub fn heroes_podium() -> CardDefinition {
    CardDefinition {
        name: "Heroes' Podium",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Each legendary creature you control gets +1/+1 for each other legendary creature you control.",
            effect: StaticEffect::PumpTeamByControlledPermanents {
                applies_to: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                count_filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                per_power: 1,
                per_toughness: 1,
                count_graveyard: false,
                exclude_self: true,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x()]),
            effect: Effect::LookPickToHand {
                then_if_picked: None,
                who: PlayerRef::You,
                count: Value::XFromCost,
                rest_to_graveyard: false,
                pick_filter: Some(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: true,
                picked_lands_to_battlefield: false,
                rest_bottom_random: true,
                rest_to_exile: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
