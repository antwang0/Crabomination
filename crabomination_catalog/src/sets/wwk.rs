//! Worldwake (WWK) — 2010. Allies, landfall, multikicker and the Zendikon
//! land animations. Tests in `classic_sets/wwk`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, Value,
};
use crate::effect::shortcut::{
    etb, landfall, on_attack, rally, target_filtered,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

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

fn spell(name: &'static str, c: crate::mana::ManaCost, ty: CardType, e: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![ty], effect: e, ..Default::default() }
}

// ── Vanilla / keyword-only ──────────────────────────────────────────────────

/// Battle Hurda — {4}{W} 3/3 Giant with first strike.
pub fn battle_hurda() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..creature("Battle Hurda", cost(&[generic(4), w()]), vec![CreatureType::Giant], 3, 3)
    }
}

/// Goliath Sphinx — {5}{U}{U} 8/7 Sphinx with flying.
pub fn goliath_sphinx() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature(
            "Goliath Sphinx",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Sphinx],
            8,
            7,
        )
    }
}

/// Grappler Spider — {1}{G} 2/1 Spider with reach.
pub fn grappler_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..creature("Grappler Spider", cost(&[generic(1), g()]), vec![CreatureType::Spider], 2, 1)
    }
}

/// Jagwasp Swarm — {3}{B} 3/2 Insect with flying.
pub fn jagwasp_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Jagwasp Swarm", cost(&[generic(3), b()]), vec![CreatureType::Insect], 3, 2)
    }
}

/// Marsh Threader — {1}{W} 2/1 Kor Scout with swampwalk.
pub fn marsh_threader() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..creature(
            "Marsh Threader",
            cost(&[generic(1), w()]),
            vec![CreatureType::Kor, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Calcite Snapper — {1}{U}{U} 1/4 Turtle with shroud. Landfall: you may switch
/// its power and toughness until end of turn.
pub fn calcite_snapper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Switch Calcite Snapper's power and toughness?".into(),
            body: Box::new(Effect::SwitchPT {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            }),
        })],
        ..creature(
            "Calcite Snapper",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Turtle],
            1,
            4,
        )
    }
}

// ── Landfall ────────────────────────────────────────────────────────────────

/// Fledgling Griffin — {1}{W} 2/2 Griffin. Landfall: gains flying EOT.
pub fn fledgling_griffin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Fledgling Griffin", cost(&[generic(1), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Hedron Rover — {4} 2/2 Construct. Landfall: +2/+2 until end of turn.
pub fn hedron_rover() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![landfall(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Hedron Rover", cost(&[generic(4)]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Caustic Crawler — {3}{B}{B} 4/3 Insect. Landfall: you may give a creature
/// -1/-1 until end of turn.
pub fn caustic_crawler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Give target creature -1/-1?".into(),
            body: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..creature(
            "Caustic Crawler",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Insect],
            4,
            3,
        )
    }
}

/// Cosi's Ravager — {3}{R} 2/2 Elemental. Landfall: you may ping a player.
pub fn cosis_ravager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Deal 1 damage to target player?".into(),
            body: Box::new(Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
            }),
        })],
        ..creature("Cosi's Ravager", cost(&[generic(3), r()]), vec![CreatureType::Elemental], 2, 2)
    }
}

/// Seer's Sundial — {4} Artifact. Landfall: pay {2} to draw a card.
pub fn seers_sundial() -> CardDefinition {
    CardDefinition {
        name: "Seer's Sundial",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![landfall(Effect::MayPay {
            description: "Pay {2} to draw a card?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(crate::effect::shortcut::draw(1)),
            else_: None,
        })],
        ..Default::default()
    }
}

// ── Allies ──────────────────────────────────────────────────────────────────

/// The WWK "put a +1/+1 counter on this" Ally shape.
fn rally_self_counter(base: CardDefinition, may: bool) -> CardDefinition {
    let body = Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        triggered_abilities: vec![rally(if may {
            Effect::MayDo {
                description: "Put a +1/+1 counter on this creature?".into(),
                body: Box::new(body),
            }
        } else {
            body
        })],
        ..base
    }
}

/// Hada Freeblade — {W} 0/1 Human Soldier Ally. Rally: a +1/+1 counter.
pub fn hada_freeblade() -> CardDefinition {
    rally_self_counter(
        ally("Hada Freeblade", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 0, 1),
        true,
    )
}

/// Bojuka Brigand — {1}{B} 1/1 Human Warrior Ally that can't block. Rally: a
/// +1/+1 counter.
pub fn bojuka_brigand() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        ..rally_self_counter(
            ally(
                "Bojuka Brigand",
                cost(&[generic(1), b()]),
                vec![CreatureType::Human, CreatureType::Warrior],
                1,
                1,
            ),
            true,
        )
    }
}

/// Graypelt Hunter — {3}{G} 2/2 Human Warrior Ally with trample. Rally: a
/// +1/+1 counter.
pub fn graypelt_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..rally_self_counter(
            ally(
                "Graypelt Hunter",
                cost(&[generic(3), g()]),
                vec![CreatureType::Human, CreatureType::Warrior],
                2,
                2,
            ),
            true,
        )
    }
}

/// Akoum Battlesinger — {1}{R} 1/1 Human Berserker Ally with haste. Rally: your
/// Allies get +1/+0 until end of turn.
pub fn akoum_battlesinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Give your Allies +1/+0?".into(),
            body: Box::new(Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..ally(
            "Akoum Battlesinger",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            1,
            1,
        )
    }
}

/// Halimar Excavator — {1}{U} 1/3 Human Wizard Ally. Rally: a player mills one
/// per Ally you control.
pub fn halimar_excavator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::Mill {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::count(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
            )),
        })],
        ..ally(
            "Halimar Excavator",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Harabaz Druid — {1}{G} 0/1 Human Druid Ally. {T}: X mana of any one color,
/// X being the number of Allies you control.
pub fn harabaz_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::count(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                ))),
            },
            ..Default::default()
        }],
        ..ally(
            "Harabaz Druid",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            0,
            1,
        )
    }
}

/// Join the Ranks — {3}{W} Instant. Two 1/1 white Soldier Allies.
pub fn join_the_ranks() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier, CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    };
    spell(
        "Join the Ranks",
        cost(&[generic(3), w()]),
        CardType::Instant,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: soldier,
        },
    )
}

// ── Zendikon auras ──────────────────────────────────────────────────────────

/// The Zendikon cycle: an Aura that animates the enchanted land and returns it
/// to hand when it dies.
pub(crate) fn zendikon(
    name: &'static str,
    c: crate::mana::ManaCost,
    (p, t): (i32, i32),
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        triggered_abilities: vec![
            etb(Effect::BecomeCreature {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::Const(p),
                toughness: Value::Const(t),
                creature_types: types,
                keywords,
                duration: Duration::Permanent,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                },
            },
        ],
        ..Default::default()
    }
}

/// Guardian Zendikon — {2}{W}. The land becomes a 2/6 white Wall with defender.
pub fn guardian_zendikon() -> CardDefinition {
    zendikon(
        "Guardian Zendikon",
        cost(&[generic(2), w()]),
        (2, 6),
        vec![CreatureType::Wall],
        vec![Keyword::Defender],
    )
}

/// Corrupted Zendikon — {1}{B}. The land becomes a 3/3 black Ooze.
pub fn corrupted_zendikon() -> CardDefinition {
    zendikon(
        "Corrupted Zendikon",
        cost(&[generic(1), b()]),
        (3, 3),
        vec![CreatureType::Ooze],
        vec![],
    )
}

/// Crusher Zendikon — {2}{R}. The land becomes a 4/2 red Beast with trample.
pub fn crusher_zendikon() -> CardDefinition {
    zendikon(
        "Crusher Zendikon",
        cost(&[generic(2), r()]),
        (4, 2),
        vec![CreatureType::Beast],
        vec![Keyword::Trample],
    )
}

// ── Lands ───────────────────────────────────────────────────────────────────

pub(crate) fn tapped_etb_land(
    name: &'static str,
    color: Color,
    trigger: Effect,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(trigger)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![color]) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Halimar Depths — enters tapped; ETB rearranges the top three.
pub fn halimar_depths() -> CardDefinition {
    tapped_etb_land(
        "Halimar Depths",
        Color::Blue,
        Effect::RearrangeTop { who: PlayerRef::You, amount: Value::Const(3) },
    )
}

/// Khalni Garden — enters tapped; ETB makes a 0/1 green Plant.
pub fn khalni_garden() -> CardDefinition {
    let plant = TokenDefinition {
        name: "Plant".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        ..Default::default()
    };
    tapped_etb_land(
        "Khalni Garden",
        Color::Green,
        Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: plant },
    )
}

/// Dread Statuary — {T}: {C}; {4}: becomes a 4/2 Golem artifact creature until
/// end of turn, still a land.
pub fn dread_statuary() -> CardDefinition {
    CardDefinition {
        name: "Dread Statuary",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: crate::effect::shortcut::add_colorless(1),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(2),
                    creature_types: vec![CreatureType::Golem],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Bull Rush — {R} Instant. Target creature gets +2/+0 until end of turn.
pub fn bull_rush() -> CardDefinition {
    spell(
        "Bull Rush",
        cost(&[r()]),
        CardType::Instant,
        crate::effect::shortcut::pump_target(2, 0),
    )
}

/// Iona's Judgment — {4}{W} Sorcery. Exile target creature or enchantment.
pub fn ionas_judgment() -> CardDefinition {
    spell(
        "Iona's Judgment",
        cost(&[generic(4), w()]),
        CardType::Sorcery,
        Effect::Exile { what: target_filtered(R::Creature.or(R::Enchantment)) },
    )
}

/// Aether Tradewinds — {2}{U} Instant. Bounce one of yours and one of theirs.
pub fn aether_tradewinds() -> CardDefinition {
    spell(
        "Aether Tradewinds",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Permanent.and(R::ControlledByOpponent),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
    )
}

/// Roiling Terrain — {2}{R}{R} Sorcery. Destroy a land, then hit its controller
/// for the land cards in their graveyard.
pub fn roiling_terrain() -> CardDefinition {
    spell(
        "Roiling Terrain",
        cost(&[generic(2), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Land) },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::CardsInGraveyardMatching {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    filter: R::Land,
                },
            },
        ]),
    )
}

/// Selective Memory — {3}{U} Sorcery. Exile any number of nonland cards from
/// your library, then shuffle.
pub fn selective_memory() -> CardDefinition {
    spell(
        "Selective Memory",
        cost(&[generic(3), u()]),
        CardType::Sorcery,
        Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Nonland,
            to: ZoneDest::Exile,
            count: Value::Const(60),
        },
    )
}

/// Rest for the Weary — {1}{W} Instant. Gain 4 life, or 8 with landfall.
pub fn rest_for_the_weary() -> CardDefinition {
    spell(
        "Rest for the Weary",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::LandsPlayedThisTurn(PlayerRef::You),
                Value::Const(1),
            ),
            then: Box::new(Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(8),
            }),
            else_: Box::new(Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(4),
            }),
        },
    )
}

// ── Equipment & misc permanents ─────────────────────────────────────────────

/// Kitesail — {2} Equipment. +1/+0 and flying. Equip {2}.
pub fn kitesail() -> CardDefinition {
    CardDefinition {
        name: "Kitesail",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Kitesail Apprentice — {W} 1/1 Kor Soldier: +1/+1 and flying while equipped.
pub fn kitesail_apprentice() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "While equipped, this creature gets +1/+1 and has flying.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsEquipped,
                },
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..creature(
            "Kitesail Apprentice",
            cost(&[w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Sejiri Merfolk — {1}{U} 2/1 Merfolk Soldier: first strike and lifelink while
/// you control a Plains.
pub fn sejiri_merfolk() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Has first strike and lifelink as long as you control a Plains.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Plains).and(R::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
            },
        }],
        ..creature(
            "Sejiri Merfolk",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            2,
            1,
        )
    }
}

/// Novablast Wurm — {3}{G}{G}{W}{W} 7/7 Wurm. Attacks: destroy all other
/// creatures.
pub fn novablast_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
        })],
        ..creature(
            "Novablast Wurm",
            cost(&[generic(3), g(), g(), w(), w()]),
            vec![CreatureType::Wurm],
            7,
            7,
        )
    }
}

/// Anowon, the Ruin Sage — {3}{B}{B} 4/3 Vampire Shaman. Upkeep: each player
/// sacrifices a non-Vampire creature.
pub fn anowon_the_ruin_sage() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::Const(1),
                filter: R::Creature.and(R::Not(Box::new(R::HasCreatureType(
                    CreatureType::Vampire,
                )))),
            },
        }],
        ..creature(
            "Anowon, the Ruin Sage",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Shaman],
            4,
            3,
        )
    }
}

/// Enclave Elite — {2}{U} 2/2 Merfolk Soldier with islandwalk and multikicker
/// {1}{U}; enters with a +1/+1 counter per kick.
pub fn enclave_elite() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Landwalk(LandType::Island),
            Keyword::Multikicker(cost(&[generic(1), u()])),
        ],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::TimesKicked)),
        ..creature(
            "Enclave Elite",
            cost(&[generic(2), u()]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Deathforge Shaman — {4}{R} 4/3 Ogre Shaman with multikicker {R}. ETB: twice
/// the kick count in damage to a player.
pub fn deathforge_shaman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Multikicker(cost(&[r()]))],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::TimesKicked)),
        })],
        ..creature(
            "Deathforge Shaman",
            cost(&[generic(4), r()]),
            vec![CreatureType::Ogre, CreatureType::Shaman],
            4,
            3,
        )
    }
}

/// Comet Storm — {X}{R}{R} Instant with multikicker {1}. X damage to one target
/// plus one more per kick.
pub fn comet_storm() -> CardDefinition {
    CardDefinition {
        name: "Comet Storm",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Multikicker(cost(&[generic(1)]))],
        effect: Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 1,
            filter: R::Any,
            effect: Box::new(Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::XFromCost,
            }),
        },
        ..Default::default()
    }
}

/// Grotag Thrasher — {4}{R} 3/3 Lizard. Attacks: a creature can't block.
pub fn grotag_thrasher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Grotag Thrasher", cost(&[generic(4), r()]), vec![CreatureType::Lizard], 3, 3)
    }
}

/// Archon of Redemption — {3}{W}{W} 3/4 Archon with flying. Whenever it or
/// another flier you control enters, gain life equal to that creature's power.
pub fn archon_of_redemption() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..creature(
            "Archon of Redemption",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Archon],
            3,
            4,
        )
    }
}

/// Ruin Ghost — {1}{W} 1/1 Spirit. {W}, {T}: blink a land you control.
pub fn ruin_ghost() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Land.and(R::ControlledByYou)) },
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ]),
            ..Default::default()
        }],
        ..creature("Ruin Ghost", cost(&[generic(1), w()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Brink of Disaster — {2}{B}{B} Aura. Destroy the enchanted permanent when it
/// becomes tapped.
pub fn brink_of_disaster() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Brink of Disaster",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.or(R::Land)),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::EnchantedBySource),
            effect: Effect::Destroy {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Bazaar Trader — {1}{R} 1/1 Goblin. {T}: give a player one of your artifacts,
/// creatures or lands.
pub fn bazaar_trader() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControl {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: (R::Artifact.or(R::Creature).or(R::Land)).and(R::ControlledByYou),
                },
                to: Some(PlayerRef::Target(0)),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..creature("Bazaar Trader", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Claws of Valakut — {1}{R}{R} Aura. +1/+0 per Mountain you control and first
/// strike.
pub fn claws_of_valakut() -> CardDefinition {
    CardDefinition {
        name: "Claws of Valakut",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::FirstStrike],
            scale: Some(crate::card::EquipScale {
                filter: R::HasLandType(LandType::Mountain),
                per_power: 1,
                per_toughness: 0,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Canopy Cover — {1}{G} Aura. The enchanted creature can only be blocked by
/// fliers/reach and can't be targeted by opponents.
pub fn canopy_cover() -> CardDefinition {
    CardDefinition {
        name: "Canopy Cover",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![
                Keyword::CantBeBlockedExceptBy(Box::new(
                    R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Reach)),
                )),
                Keyword::Hexproof,
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}
