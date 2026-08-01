//! Planeshift (PLS) closure — the Familiars, the bounce cycle, the Battlemages
//! and the rest of the set. Tests in `classic_sets/pls2`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{deal, draw, etb, target_any, target_filtered},
};
use crate::game::TurnStep;
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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..enchantment(name, c)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// "As long as you control a [filter]".
fn you_control(filter: R) -> Predicate {
    Predicate::SelectorExists(Selector::EachPermanent(filter.and(R::ControlledByYou)))
}

/// 1/1 green Saproling.
fn saproling() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The Invasion-block bounce riders: "return a [colors] creature you control to
/// its owner's hand."
fn bounce_own(a: Color, b: Color) -> TriggeredAbility {
    etb(Effect::Move {
        what: target_filtered(
            R::Creature.and(R::ControlledByYou).and(R::HasColor(a).or(R::HasColor(b))),
        ),
        to: ZoneDest::Hand(PlayerRef::You),
    })
}

/// The Familiar cycle: "[A] spells and [B] spells you cast cost {1} less."
fn familiar_discount(a: Color, b: Color, description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::CostReduction {
            filter: R::HasColor(a).or(R::HasColor(b)),
            amount: 1,
        },
    }
}

/// Doomsday Specter — {2}{U}{B} 2/3 flier that bounces one of yours, then picks
/// the discard.
pub fn doomsday_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            bounce_own(Color::Blue, Color::Black),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::SelfSource,
                ),
                effect: Effect::DiscardChosen {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Any,
                },
            },
        ],
        ..creature(
            "Doomsday Specter",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Specter],
            2,
            3,
        )
    }
}

/// Mana Cylix — {1}. The worst filter ever printed.
pub fn mana_cylix() -> CardDefinition {
    CardDefinition {
        name: "Mana Cylix",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mire Kavu — {3}{R} 3/2 that grows beside a Swamp.
pub fn mire_kavu() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Mire Kavu gets +1/+1 as long as you control a Swamp.",
            effect: StaticEffect::PumpSelfIf {
                condition: you_control(R::HasLandType(LandType::Swamp)),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..creature("Mire Kavu", cost(&[generic(3), r()]), vec![CreatureType::Kavu], 3, 2)
    }
}

/// Mogg Sentry — {R} 1/1 that swells whenever an opponent casts.
pub fn mogg_sentry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Mogg Sentry",
            cost(&[r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Morgue Toad — {2}{B} 2/2 that cashes out for {U}{R}.
pub fn morgue_toad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Blue, Color::Red]),
            },
            ..Default::default()
        }],
        ..creature("Morgue Toad", cost(&[generic(2), b()]), vec![CreatureType::Frog], 2, 2)
    }
}

/// Nemata, Grove Guardian — {4}{G}{G} 4/5. Makes Saprolings, eats them for a
/// team pump.
pub fn nemata_grove_guardian() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: saproling(),
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Saproling), 1)),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::HasCreatureType(CreatureType::Saproling)),
                    ),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Nemata, Grove Guardian",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Treefolk],
            4,
            5,
        )
    }
}

/// Nightscape Familiar — {1}{B} 1/1 Grixis-half discounter that regenerates.
pub fn nightscape_familiar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![familiar_discount(
            Color::Blue,
            Color::Red,
            "Blue spells and red spells you cast cost {1} less to cast.",
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Nightscape Familiar", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 1, 1)
    }
}

/// Stormscape Familiar — {1}{U} 1/1 flier discounting white and black.
pub fn stormscape_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![familiar_discount(
            Color::White,
            Color::Black,
            "White spells and black spells you cast cost {1} less to cast.",
        )],
        ..creature("Stormscape Familiar", cost(&[generic(1), u()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Sunscape Familiar — {1}{W} 0/3 Wall discounting green and blue.
pub fn sunscape_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        static_abilities: vec![familiar_discount(
            Color::Green,
            Color::Blue,
            "Green spells and blue spells you cast cost {1} less to cast.",
        )],
        ..creature("Sunscape Familiar", cost(&[generic(1), w()]), vec![CreatureType::Wall], 0, 3)
    }
}

/// Thornscape Familiar — {1}{G} 2/1 discounting red and white.
pub fn thornscape_familiar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![familiar_discount(
            Color::Red,
            Color::White,
            "Red spells and white spells you cast cost {1} less to cast.",
        )],
        ..creature("Thornscape Familiar", cost(&[generic(1), g()]), vec![CreatureType::Insect], 2, 1)
    }
}

/// Thunderscape Familiar — {1}{R} 1/1 first striker discounting black and green.
pub fn thunderscape_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![familiar_discount(
            Color::Black,
            Color::Green,
            "Black spells and green spells you cast cost {1} less to cast.",
        )],
        ..creature("Thunderscape Familiar", cost(&[generic(1), r()]), vec![CreatureType::Kavu], 1, 1)
    }
}

/// Shriek of Dread — {1}{B}. Fear for a turn.
pub fn shriek_of_dread() -> CardDefinition {
    instant(
        "Shriek of Dread",
        cost(&[generic(1), b()]),
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Fear,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Silver Drake — {1}{W}{U} 3/3 flier with the Invasion bounce rider.
pub fn silver_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![bounce_own(Color::White, Color::Blue)],
        ..creature(
            "Silver Drake",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Drake],
            3,
            3,
        )
    }
}

/// Shivan Wurm — {3}{R}{G} 7/7 trampler with the bounce rider.
pub fn shivan_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![bounce_own(Color::Red, Color::Green)],
        ..creature("Shivan Wurm", cost(&[generic(3), r(), g()]), vec![CreatureType::Wurm], 7, 7)
    }
}

/// Steel Leaf Paladin — {4}{G}{W} 4/4 first striker with the bounce rider.
pub fn steel_leaf_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![bounce_own(Color::Green, Color::White)],
        ..creature(
            "Steel Leaf Paladin",
            cost(&[generic(4), g(), w()]),
            vec![CreatureType::Elf, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Sparkcaster — {2}{R}{G} 5/3 that bounces one of yours and pings a player.
pub fn sparkcaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            bounce_own(Color::Red, Color::Green),
            etb(deal(1, target_filtered(R::Player.or(R::Planeswalker)))),
        ],
        ..creature("Sparkcaster", cost(&[generic(2), r(), g()]), vec![CreatureType::Kavu], 5, 3)
    }
}

/// Marsh Crocodile — {2}{U}{B} 4/4 that bounces one of yours and strips a card.
pub fn marsh_crocodile() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            bounce_own(Color::Blue, Color::Black),
            etb(Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
                random: false,
            }),
        ],
        ..creature(
            "Marsh Crocodile",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Crocodile],
            4,
            4,
        )
    }
}

/// Razing Snidd — {4}{B}{R} 3/3 that bounces one of yours and Armageddons a land.
pub fn razing_snidd() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            bounce_own(Color::Black, Color::Red),
            etb(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::ONE,
                filter: R::Land,
            }),
        ],
        ..creature("Razing Snidd", cost(&[generic(4), b(), r()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Pygmy Kavu — {3}{G} 1/2 that draws off the opponents' black creatures.
pub fn pygmy_kavu() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::count(Selector::EachPermanent(
                R::Creature.and(R::ControlledByOpponent).and(R::HasColor(Color::Black)),
            )),
        })],
        ..creature("Pygmy Kavu", cost(&[generic(3), g()]), vec![CreatureType::Kavu], 1, 2)
    }
}

/// Volcano Imp — {3}{B} 2/2 flier with a red first-strike pump.
pub fn volcano_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Volcano Imp", cost(&[generic(3), b()]), vec![CreatureType::Imp], 2, 2)
    }
}

/// Stone Kavu — {4}{G} 3/3 with a red and a white pump.
pub fn stone_kavu() -> CardDefinition {
    let pump = |symbol, power, toughness| ActivatedAbility {
        mana_cost: cost(&[symbol]),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![pump(r(), 1, 0), pump(w(), 0, 1)],
        ..creature("Stone Kavu", cost(&[generic(4), g()]), vec![CreatureType::Kavu], 3, 3)
    }
}

/// Strafe — {R}. Three damage, but not to red.
pub fn strafe() -> CardDefinition {
    sorcery(
        "Strafe",
        cost(&[r()]),
        deal(3, target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Red)))))),
    )
}

/// Singe — {R}. A ping that paints the victim black.
pub fn singe() -> CardDefinition {
    instant(
        "Singe",
        cost(&[r()]),
        Effect::Seq(vec![
            deal(1, target_filtered(R::Creature)),
            Effect::BecomeColor {
                what: target_filtered(R::Creature),
                colors: vec![Color::Black],
                duration: Duration::EndOfTurn,
                additive: false,
            },
        ]),
    )
}

/// Sinister Strength — {1}{B} Aura. +3/+1 and black.
pub fn sinister_strength() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 1, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is black.",
            effect: StaticEffect::SetColorOfMatching {
                applies_to: Selector::attached_to(Selector::This),
                color: Color::Black,
            },
        }],
        ..aura("Sinister Strength", cost(&[generic(1), b()]))
    }
}

/// Slay — {2}{B}. Kills green and replaces itself.
pub fn slay() -> CardDefinition {
    instant(
        "Slay",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Green))),
            },
            draw(1),
        ]),
    )
}

/// Slingshot Goblin — {2}{R} 2/2 that snipes blue creatures.
pub fn slingshot_goblin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            effect: deal(2, target_filtered(R::Creature.and(R::HasColor(Color::Blue)))),
            ..Default::default()
        }],
        ..creature("Slingshot Goblin", cost(&[generic(2), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

/// Terminal Moraine — a colorless land that cracks for a basic.
pub fn terminal_moraine() -> CardDefinition {
    CardDefinition {
        name: "Terminal Moraine",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::IsBasicLand),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Urza's Guilt — {2}{U}{B}. Everyone draws two, pitches three and bleeds.
pub fn urzas_guilt() -> CardDefinition {
    sorcery(
        "Urza's Guilt",
        cost(&[generic(2), u(), b()]),
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(3),
                random: false,
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(4),
            },
        ]),
    )
}

/// The Lair cycle's last two members.
fn lair(name: &'static str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Lair], ..Default::default() },
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessReturn {
            filter: R::Land.and(R::Not(Box::new(R::HasLandType(LandType::Lair)))),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(colors, Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rith's Grove — the Naya Lair.
pub fn riths_grove() -> CardDefinition {
    lair("Rith's Grove", vec![Color::Red, Color::Green, Color::White])
}

/// Treva's Ruins — the Bant Lair.
pub fn trevas_ruins() -> CardDefinition {
    lair("Treva's Ruins", vec![Color::Green, Color::White, Color::Blue])
}

/// Phyrexian Bloodstock — {4}{B} 3/3 whose departure kills a white creature.
pub fn phyrexian_bloodstock() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasColor(Color::White))),
            },
        }],
        ..creature(
            "Phyrexian Bloodstock",
            cost(&[generic(4), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Phyrexian Scuta — {3}{B} 3/3. Kicker—Pay 3 life for two counters.
pub fn phyrexian_scuta() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(ManaCost::default())],
        kicker_action_cost: Some(AdditionalCastCost::PayLife { amount: 3 }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Phyrexian Scuta",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Mogg Jailer — {1}{R} 2/2 that small untapped blockers keep at home.
pub fn mogg_jailer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Mogg Jailer can't attack if defending player controls an untapped \
                          creature with power 2 or less.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByOpponent)
                        .and(R::Untapped)
                        .and(R::PowerAtMost(2)),
                )),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::CantAttack],
            },
        }],
        ..creature("Mogg Jailer", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

/// Domain — "this spell costs {per} less to cast for each basic land type among
/// lands you control."
fn domain_discount(per: u32) -> Option<(Value, u32)> {
    Some((Value::DomainCount(PlayerRef::You), per))
}

/// Draco — {16} 9/9 flier. Domain pays for it, and Domain keeps it alive.
pub fn draco() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        self_cost_reduction_per: domain_discount(2),
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPayValue {
                generic: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::Const(10)),
                    Box::new(Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::DomainCount(PlayerRef::You)),
                    )),
                ))),
            },
        }],
        ..creature("Draco", cost(&[generic(16)]), vec![CreatureType::Dragon], 9, 9)
    }
}

/// Stratadon — {10} 5/5 trampler that Domain drags into range.
pub fn stratadon() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        self_cost_reduction_per: domain_discount(1),
        keywords: vec![Keyword::Trample],
        ..creature("Stratadon", cost(&[generic(10)]), vec![CreatureType::Beast], 5, 5)
    }
}

/// Magnigoth Treefolk — {4}{G} 2/6 with one landwalk per basic type you control.
pub fn magnigoth_treefolk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DomainLandwalk],
        ..creature("Magnigoth Treefolk", cost(&[generic(4), g()]), vec![CreatureType::Treefolk], 2, 6)
    }
}

/// Lashknife Barrier — {2}{W}. Cantrips, then shaves a point off every hit.
pub fn lashknife_barrier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage to a creature you control, it deals that \
                          much damage minus 1 to that creature instead.",
            effect: StaticEffect::ReduceDamageToYourCreaturesBy(1),
        }],
        ..enchantment("Lashknife Barrier", cost(&[generic(2), w()]))
    }
}

/// Sawtooth Loon — {2}{W}{U} 2/2 flier that bounces one of yours and filters two.
pub fn sawtooth_loon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            bounce_own(Color::White, Color::Blue),
            etb(Effect::Seq(vec![
                draw(2),
                Effect::PutCardsFromHandOnBottom {
                    who: Selector::You,
                    count: Value::Const(2),
                },
            ])),
        ],
        ..creature("Sawtooth Loon", cost(&[generic(2), w(), u()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Shifting Sky — {2}{U}. Repaints the whole board.
pub fn shifting_sky() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "All nonland permanents are the chosen color.",
            effect: StaticEffect::SetColorOfMatchingToChosen {
                applies_to: Selector::EachPermanent(R::Nonland),
            },
        }],
        ..enchantment("Shifting Sky", cost(&[generic(2), u()]))
    }
}

/// Voice of All — {2}{W}{W} 2/2 flier with protection from a chosen color.
pub fn voice_of_all() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "Voice of All has protection from the chosen color.",
            effect: StaticEffect::GrantProtectionFromChosenColor { applies_to: Selector::This },
        }],
        ..creature("Voice of All", cost(&[generic(2), w(), w()]), vec![CreatureType::Angel], 2, 2)
    }
}

/// The Planeswalker's cycle: "{3}{C}: Target opponent reveals a card at random
/// from their hand", then a payoff scaled by that card's mana value.
fn planeswalkers_enchantment(
    name: &'static str,
    c: ManaCost,
    activation: ManaCost,
    sorcery_speed: bool,
    payoff: Effect,
) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: activation,
            sorcery_speed,
            effect: Effect::Seq(vec![
                Effect::RevealRandomFromHand {
                    who: target_filtered(R::OpponentPlayer),
                },
                payoff,
            ]),
            ..Default::default()
        }],
        ..enchantment(name, c)
    }
}

/// Planeswalker's Favor — {2}{G}. Pumps by the revealed card's mana value.
pub fn planeswalkers_favor() -> CardDefinition {
    planeswalkers_enchantment(
        "Planeswalker's Favor",
        cost(&[generic(2), g()]),
        cost(&[generic(3), g()]),
        false,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::LastRevealedManaValue,
            toughness: Value::LastRevealedManaValue,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Planeswalker's Fury — {2}{R}. Burns by the revealed card's mana value.
pub fn planeswalkers_fury() -> CardDefinition {
    planeswalkers_enchantment(
        "Planeswalker's Fury",
        cost(&[generic(2), r()]),
        cost(&[generic(3), r()]),
        true,
        Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::LastRevealedManaValue,
        },
    )
}

/// Planeswalker's Mirth — {2}{W}. Gains by the revealed card's mana value.
pub fn planeswalkers_mirth() -> CardDefinition {
    planeswalkers_enchantment(
        "Planeswalker's Mirth",
        cost(&[generic(2), w()]),
        cost(&[generic(3), w()]),
        false,
        Effect::GainLife { who: Selector::You, amount: Value::LastRevealedManaValue },
    )
}

/// Planeswalker's Scorn — {2}{B}. Shrinks by the revealed card's mana value.
pub fn planeswalkers_scorn() -> CardDefinition {
    planeswalkers_enchantment(
        "Planeswalker's Scorn",
        cost(&[generic(2), b()]),
        cost(&[generic(3), b()]),
        true,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::LastRevealedManaValue)),
            toughness: Value::Times(
                Box::new(Value::Const(-1)),
                Box::new(Value::LastRevealedManaValue),
            ),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Malicious Advice — {X}{U}{B}. Taps X things and bills you for it.
pub fn malicious_advice() -> CardDefinition {
    instant(
        "Malicious Advice",
        cost(&[crate::mana::x(), u(), b()]),
        Effect::Seq(vec![
            Effect::TapUpToValue {
                count: Value::XFromCost,
                filter: R::Artifact.or(R::Creature).or(R::Land),
                skip_untap: false,
                exact: false,
            },
            Effect::LoseLife { who: Selector::You, amount: Value::XFromCost },
        ]),
    )
}

/// Skyshroud Blessing — {1}{G}. Lands go untargetable and you cantrip.
pub fn skyshroud_blessing() -> CardDefinition {
    instant(
        "Skyshroud Blessing",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::GrantKeywordToMatchingThisTurn { filter: R::Land, keyword: Keyword::Shroud },
            draw(1),
        ]),
    )
}

/// March of Souls — {4}{W}. A wrath that hands everyone Spirits back.
pub fn march_of_souls() -> CardDefinition {
    sorcery(
        "March of Souls",
        cost(&[generic(4), w()]),
        Effect::DestroyThenVictimControllersMakeToken {
            what: Selector::EachPermanent(R::Creature),
            definition: crate::card::TokenDefinition {
                name: "Spirit".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                keywords: vec![Keyword::Flying],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Spirit],
                    ..Default::default()
                },
                ..Default::default()
            },
            no_regen: true,
        },
    )
}

/// Sunken Hope — {3}{U}{U}. Everyone re-buys a creature every upkeep.
pub fn sunken_hope() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::EachPlayerReturnsAMatchingPermanent { filter: R::Creature },
        }],
        ..enchantment("Sunken Hope", cost(&[generic(3), u(), u()]))
    }
}

/// Warped Devotion — {2}{B}. Every bounce costs a card.
pub fn warped_devotion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentReturnedToHand, EventScope::AnyPlayer),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..enchantment("Warped Devotion", cost(&[generic(2), b()]))
    }
}

/// Natural Emergence — {2}{R}{G}. Lands stand up and fight.
pub fn natural_emergence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::Enchantment
                    .and(R::ControlledByYou)
                    .and(R::HasColor(Color::Red).or(R::HasColor(Color::Green))),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        static_abilities: vec![StaticAbility {
            description: "Lands you control are 2/2 creatures with first strike. They're still \
                          lands.",
            effect: StaticEffect::MatchingLandsAreCreatures {
                filter: R::Land.and(R::ControlledByYou),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        ..enchantment("Natural Emergence", cost(&[generic(2), r(), g()]))
    }
}

/// Rushing River — {2}{U}. Kicker—Sacrifice a land for a second bounce.
pub fn rushing_river() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }),
        ..instant(
            "Rushing River",
            cost(&[generic(2), u()]),
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Permanent.and(R::Nonland)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Move {
                        what: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::Permanent.and(R::Nonland),
                        },
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(1)))),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Magma Burst — {3}{R}. Kicker—Sacrifice two lands for a second bolt.
pub fn magma_burst() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 2,
        }),
        ..instant(
            "Magma Burst",
            cost(&[generic(3), r()]),
            Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::DealDamage {
                        to: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::Creature.or(R::Player).or(R::Planeswalker),
                        },
                        amount: Value::Const(3),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Primal Growth — {2}{G}. Kicker—Sacrifice a creature for a second land.
pub fn primal_growth() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }),
        ..sorcery(
            "Primal Growth",
            cost(&[generic(2), g()]),
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Land.and(R::IsBasicLand),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                count: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(2)),
                    else_: Box::new(Value::ONE),
                },
            },
        )
    }
}

/// Pollen Remedy — {W}. Kicker—Sacrifice a land to double the shield.
pub fn pollen_remedy() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }),
        ..instant(
            "Pollen Remedy",
            cost(&[w()]),
            Effect::PreventNextDamageDivided {
                total: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(6)),
                    else_: Box::new(Value::Const(3)),
                },
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
                max_targets: 3,
            },
        )
    }
}

/// Waterspout Elemental — {3}{U}{U} 3/4 flier. Kicked, it resets the board and
/// costs you a turn.
pub fn waterspout_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[u()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::SkipTurns { who: PlayerRef::You, count: Value::ONE },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Waterspout Elemental",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Elemental],
            3,
            4,
        )
    }
}

/// Rith's Charm — {R}{G}{W}. Land destruction, Saprolings, or a fog.
pub fn riths_charm() -> CardDefinition {
    instant(
        "Rith's Charm",
        cost(&[r(), g(), w()]),
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Land.and(R::IsNonbasicLand)) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: saproling(),
            },
            Effect::PreventAllDamageFromChosenSourceThisTurn { filter: R::Permanent },
        ]),
    )
}

/// Treva's Charm — {G}{W}{U}. Naturalize, exile an attacker, or loot.
pub fn trevas_charm() -> CardDefinition {
    instant(
        "Treva's Charm",
        cost(&[g(), w(), u()]),
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            Effect::Exile { what: target_filtered(R::Creature.and(R::IsAttacking)) },
            Effect::Seq(vec![
                draw(1),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        ]),
    )
}

/// Root Greevil — {3}{G} 2/3 that eats a colour's worth of enchantments.
pub fn root_greevil() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::ChooseColorForSelf,
                Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::Enchantment.and(R::HasChosenColorOfSource),
                    ),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Root Greevil", cost(&[generic(3), g()]), vec![CreatureType::Beast], 2, 3)
    }
}

/// Phyrexian Tyranny — {U}{B}{R}. Card draw hurts.
pub fn phyrexian_tyranny() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::AnyPlayer),
            effect: Effect::MayPayBy {
                who: PlayerRef::TriggerEventPlayer,
                description: "Pay {2} or lose 2 life?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                })),
            },
        }],
        ..enchantment("Phyrexian Tyranny", cost(&[u(), b(), r()]))
    }
}

/// Sea Snidd — {4}{U} 3/3 that rewrites a land's type.
pub fn sea_snidd() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LandsBecomeChosenBasicType {
                what: target_filtered(R::Land),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Sea Snidd", cost(&[generic(4), u()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Quirion Explorer — {1}{G} 1/1 that taps for whatever the opponents can make.
pub fn quirion_explorer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColorOpponentCouldProduce,
            },
            ..Default::default()
        }],
        ..creature(
            "Quirion Explorer",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Druid, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Forsaken City — a land that stays tapped unless you feed it a card.
pub fn forsaken_city() -> CardDefinition {
    CardDefinition {
        name: "Forsaken City",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "Forsaken City doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exile a card from your hand to untap Forsaken City?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::ExileChosenFromHand {
                        from: Selector::You,
                        count: Value::ONE,
                        filter: R::Any,
                        link_to_source: false,
                        face_down: false,
                    },
                    Effect::Untap { what: Selector::This, up_to: None },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sleeping Potion — {1}{U} Aura. Taps a creature down until someone pokes it.
pub fn sleeping_potion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Tap { what: Selector::attached_to(Selector::This) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::EnchantedBySource),
                effect: Effect::SacrificeSource,
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::attached_to(Selector::This),
            },
        }],
        ..aura("Sleeping Potion", cost(&[generic(1), u()]))
    }
}

/// Multani's Harmony — {G} Aura. Turns a creature into a mana rock.
pub fn multanis_harmony() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Multani's Harmony", cost(&[g()]))
    }
}

/// Sisay's Ingenuity — {U} Aura. Cantrips, then lends a recolouring ability.
pub fn sisays_ingenuity() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                effect: Effect::BecomeChosenColor {
                    what: target_filtered(R::Creature),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Sisay's Ingenuity", cost(&[u()]))
    }
}

/// Radiant Kavu — {R}{G}{W} 3/3 that blanks the blue-black swing.
pub fn radiant_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), g(), w()]),
            effect: Effect::PreventAllCombatDamageByMatchingThisTurn {
                filter: R::Creature
                    .and(R::HasColor(Color::Blue).or(R::HasColor(Color::Black))),
            },
            ..Default::default()
        }],
        ..creature("Radiant Kavu", cost(&[r(), g(), w()]), vec![CreatureType::Kavu], 3, 3)
    }
}

/// Tahngarth, Talruum Hero — {3}{R}{R} 4/4. Trades punches on demand.
pub fn tahngarth_talruum_hero() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            effect: Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Tahngarth, Talruum Hero",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            4,
            4,
        )
    }
}
