//! Strixhaven base-set (STX) cards that were still missing from the
//! catalog — real printed cards wired against existing engine
//! primitives. Lands (Campus cycle, Access Tunnel, Archway Commons),
//! keyword creatures, and a spread of spells. Each ships with a
//! functionality test in `crate::tests::stx`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, Selector, SelectionRequirement, Subtypes,
    TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{
    etb, etb_gain_life, magecraft, mint_pests, on_attack, pump_target, target, target_filtered,
};
use crate::effect::{
    Duration, LibraryPosition, PlayerRef, StaticAbility, StaticEffect, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

// ── Campus land cycle ─────────────────────────────────────────────────────

/// Build a Strixhaven Campus land: enters tapped, taps for one of two
/// colors, and `{4}, {T}: Scry 1`.
fn campus_land(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
) -> CardDefinition {
    use super::super::{etb_tap, tap_add};
    let scry = ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(4)]),
        effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ..Default::default()
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![type_a, type_b], ..Default::default() },
        activated_abilities: vec![tap_add(color_a), tap_add(color_b), scry],
        triggered_abilities: vec![etb_tap()],
        ..Default::default()
    }
}

/// Lorehold Campus — R/W Campus land.
pub fn lorehold_campus() -> CardDefinition {
    campus_land("Lorehold Campus", LandType::Mountain, LandType::Plains, Color::Red, Color::White)
}
/// Prismari Campus — U/R Campus land.
pub fn prismari_campus() -> CardDefinition {
    campus_land("Prismari Campus", LandType::Island, LandType::Mountain, Color::Blue, Color::Red)
}
/// Quandrix Campus — G/U Campus land.
pub fn quandrix_campus() -> CardDefinition {
    campus_land("Quandrix Campus", LandType::Forest, LandType::Island, Color::Green, Color::Blue)
}
/// Silverquill Campus — W/B Campus land.
pub fn silverquill_campus() -> CardDefinition {
    campus_land("Silverquill Campus", LandType::Plains, LandType::Swamp, Color::White, Color::Black)
}
/// Witherbloom Campus — B/G Campus land.
pub fn witherbloom_campus() -> CardDefinition {
    campus_land("Witherbloom Campus", LandType::Swamp, LandType::Forest, Color::Black, Color::Green)
}

/// Access Tunnel — `{T}: Add {3}`; `{3}, {T}: Target creature with power
/// 3 or less can't be blocked this turn.`
pub fn access_tunnel() -> CardDefinition {
    use super::super::tap_add_colorless;
    let evasion = ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(3)]),
        effect: Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
            ),
            keyword: Keyword::Unblockable,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Access Tunnel",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), evasion],
        ..Default::default()
    }
}

/// Archway Commons — enters tapped; "When this land enters, sacrifice it
/// unless you pay {1}"; `{T}: Add one mana of any color.`
pub fn archway_commons() -> CardDefinition {
    use super::super::{etb_tap, tap_add_any_color};
    CardDefinition {
        name: "Archway Commons",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_any_color()],
        triggered_abilities: vec![
            etb_tap(),
            etb(Effect::PayManaOrElse {
                mana_cost: cost(&[generic(1)]),
                otherwise: Box::new(Effect::SacrificeSource),
            }),
        ],
        ..Default::default()
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Moldering Karok — {2}{B}{G} 3/3 Zombie Crocodile with trample, lifelink.
pub fn moldering_karok() -> CardDefinition {
    CardDefinition {
        name: "Moldering Karok",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Crocodile],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Needlethorn Drake — {G}{U} 1/1 Drake with flying, deathtouch.
pub fn needlethorn_drake() -> CardDefinition {
    CardDefinition {
        name: "Needlethorn Drake",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drake], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Relic Sloth — {3}{R}{W} 4/4 Sloth Beast with vigilance, menace.
pub fn relic_sloth() -> CardDefinition {
    CardDefinition {
        name: "Relic Sloth",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sloth, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        ..Default::default()
    }
}

/// Waterfall Aerialist — {3}{U} 3/1 Djinn Wizard with flying, ward {2}.
pub fn waterfall_aerialist() -> CardDefinition {
    CardDefinition {
        name: "Waterfall Aerialist",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        ..Default::default()
    }
}

/// Springmane Cervin — {2}{G} 3/2 Elk. ETB: gain 2 life.
pub fn springmane_cervin() -> CardDefinition {
    CardDefinition {
        name: "Springmane Cervin",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elk], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

/// Professor of Zoomancy — {3}{G} 4/3 Bear Druid. ETB: create a 1/1 B/G
/// Pest with "When this dies, you gain 1 life."
pub fn professor_of_zoomancy() -> CardDefinition {
    CardDefinition {
        name: "Professor of Zoomancy",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(mint_pests(1))],
        ..Default::default()
    }
}

/// Scurrid Colony — {1}{G} 2/2 Squirrel with reach; gets +2/+2 while you
/// control eight or more lands.
pub fn scurrid_colony() -> CardDefinition {
    CardDefinition {
        name: "Scurrid Colony",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Squirrel], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Scurrid Colony gets +2/+2 as long as you control eight or more lands.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(8),
                },
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Wormhole Serpent — {4}{U} 3/5 Serpent. `{3}{U}: Target creature can't
/// be blocked this turn.`
pub fn wormhole_serpent() -> CardDefinition {
    CardDefinition {
        name: "Wormhole Serpent",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent], ..Default::default() },
        power: 3,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mage Hunter — {3}{B} 3/4 Horror. Whenever an opponent casts or copies
/// an instant or sorcery spell, they lose 1 life.
pub fn mage_hunter() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    CardDefinition {
        name: "Mage Hunter",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Leonin Lightscribe — {1}{W} 2/2 Cat Cleric. Magecraft — Whenever you
/// cast or copy an instant or sorcery, creatures you control get +1/+1
/// until end of turn.
pub fn leonin_lightscribe() -> CardDefinition {
    CardDefinition {
        name: "Leonin Lightscribe",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Blood Researcher — {1}{B}{G} 2/2 Vampire Druid with menace. Whenever
/// you gain life, put a +1/+1 counter on it.
pub fn blood_researcher() -> CardDefinition {
    CardDefinition {
        name: "Blood Researcher",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Arrogant Poet — {1}{B} 2/1 Human Warlock. Whenever it attacks, you may
/// pay 2 life. If you do, it gains flying until end of turn.
pub fn arrogant_poet() -> CardDefinition {
    CardDefinition {
        name: "Arrogant Poet",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Pay 2 life: Arrogant Poet gains flying until end of turn.".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Oggyar Battle-Seer — {3}{U}{R} 3/4 Ogre Shaman with haste; `{T}: Scry 1.`
pub fn oggyar_battle_seer() -> CardDefinition {
    CardDefinition {
        name: "Oggyar Battle-Seer",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Biomathematician — {1}{G}{U} 2/2 Human Wizard. ETB: create a 0/0 G/U
/// Fractal token, then put a +1/+1 counter on each Fractal you control.
pub fn biomathematician() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Biomathematician",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::ControlledByYou
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
        ]))],
        ..Default::default()
    }
}

/// Campus Guide — {2} 2/1 Golem artifact creature. ETB: you may search
/// your library for a basic land, reveal it, shuffle, and put it on top.
pub fn campus_guide() -> CardDefinition {
    CardDefinition {
        name: "Campus Guide",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        })],
        ..Default::default()
    }
}

/// Biblioplex Assistant — {4} 2/1 Gargoyle artifact creature with flying.
/// ETB: put up to one target instant or sorcery card from your graveyard
/// on top of your library.
pub fn biblioplex_assistant() -> CardDefinition {
    CardDefinition {
        name: "Biblioplex Assistant",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            }),
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        })],
        ..Default::default()
    }
}

/// Overgrown Arch — {1}{G} 0/4 Plant Wall with defender. `{T}: You gain 1
/// life.` `{2}, Sacrifice this creature: Learn.`
pub fn overgrown_arch() -> CardDefinition {
    CardDefinition {
        name: "Overgrown Arch",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Learn { who: PlayerRef::You },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Spells ───────────────────────────────────────────────────────────────

/// Expel — {2}{W} Instant. Exile target tapped creature.
pub fn expel() -> CardDefinition {
    CardDefinition {
        name: "Expel",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            ),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Crushing Disappointment — {3}{B} Instant. Each player loses 2 life. You
/// draw two cards.
pub fn crushing_disappointment() -> CardDefinition {
    CardDefinition {
        name: "Crushing Disappointment",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Essence Infusion — {1}{B} Sorcery. Put two +1/+1 counters on target
/// creature. It gains lifelink until end of turn.
pub fn essence_infusion() -> CardDefinition {
    CardDefinition {
        name: "Essence Infusion",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target(),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Professor's Warning — {B} Instant. Choose one — put a +1/+1 counter on
/// target creature; or target creature gains indestructible until end of
/// turn.
pub fn professors_warning() -> CardDefinition {
    CardDefinition {
        name: "Professor's Warning",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: target(),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Sudden Breakthrough — {1}{R} Instant. Target creature gets +2/+0 and
/// gains first strike until end of turn. Create a Treasure token.
pub fn sudden_breakthrough() -> CardDefinition {
    use crate::effect::shortcut::mint_treasures;
    CardDefinition {
        name: "Sudden Breakthrough",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            pump_target(2, 0),
            Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            mint_treasures(1),
        ]),
        ..Default::default()
    }
}

/// Thrilling Discovery — {R}{W} Sorcery. You gain 2 life. Then you may
/// discard two cards. If you do, draw three cards.
pub fn thrilling_discovery() -> CardDefinition {
    CardDefinition {
        name: "Thrilling Discovery",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::MayDo {
                description: "Discard two cards, then draw three cards.".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(2),
                        random: false,
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                ])),
            },
        ]),
        ..Default::default()
    }
}

/// Arcane Subtraction — {1}{U} Instant. Target creature gets -4/-0 until
/// end of turn. Learn.
pub fn arcane_subtraction() -> CardDefinition {
    CardDefinition {
        name: "Arcane Subtraction",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![pump_target(-4, 0), Effect::Learn { who: PlayerRef::You }]),
        ..Default::default()
    }
}

/// Exhilarating Elocution — {2}{W}{B} Sorcery. Put two +1/+1 counters on
/// target creature you control. Other creatures you control get +1/+1
/// until end of turn.
pub fn exhilarating_elocution() -> CardDefinition {
    CardDefinition {
        name: "Exhilarating Elocution",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Infuse with Vitality — {B}{G} Instant. Until end of turn, target
/// creature gains deathtouch and "When this creature dies, return it to
/// the battlefield tapped under its owner's control." You gain 2 life.
pub fn infuse_with_vitality() -> CardDefinition {
    CardDefinition {
        name: "Infuse with Vitality",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: target(),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                    effect: Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::This)),
                            tapped: true,
                        },
                    },
                }),
                duration: Duration::EndOfTurn,
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}
