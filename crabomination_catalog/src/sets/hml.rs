//! Homelands (HML) — opening wave. Tests in `classic_sets/hml`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{target_any, target_filtered},
};
use crate::game::types::TurnStep;
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

fn artifact_creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, c, types, p, t)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost, statics: Vec<StaticAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        static_abilities: statics,
        ..Default::default()
    }
}

/// A World enchantment (CR 704.5m — the legend rule for Worlds).
fn world_enchantment(
    name: &'static str,
    c: ManaCost,
    statics: Vec<StaticAbility>,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::World],
        ..enchantment(name, c, statics)
    }
}

/// One of Homelands' five "city" lands: {T} for {C}, {1}{T} for its primary
/// colour, {2}{T} for either of its two secondaries.
fn city_land(name: &'static str, primary: Color, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
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
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![primary]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![a, b], Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Lands ──────────────────────────────────────────────────────────────────

/// An-Havva Township — {C}, then {G}, then {R} or {W}.
pub fn an_havva_township() -> CardDefinition {
    city_land("An-Havva Township", Color::Green, Color::Red, Color::White)
}

/// Aysen Abbey — {C}, then {W}, then {G} or {U}.
pub fn aysen_abbey() -> CardDefinition {
    city_land("Aysen Abbey", Color::White, Color::Green, Color::Blue)
}

/// Castle Sengir — {C}, then {B}, then {U} or {R}.
pub fn castle_sengir() -> CardDefinition {
    city_land("Castle Sengir", Color::Black, Color::Blue, Color::Red)
}

/// Koskun Keep — {C}, then {R}, then {B} or {G}.
pub fn koskun_keep() -> CardDefinition {
    city_land("Koskun Keep", Color::Red, Color::Black, Color::Green)
}

/// Wizards' School — {C}, then {U}, then {W} or {B}.
pub fn wizards_school() -> CardDefinition {
    city_land("Wizards' School", Color::Blue, Color::White, Color::Black)
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Abbey Gargoyles — a flier red can't touch.
pub fn abbey_gargoyles() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        ..creature(
            "Abbey Gargoyles",
            cost(&[generic(2), w(), w(), w()]),
            vec![CreatureType::Gargoyle],
            3,
            4,
        )
    }
}

/// Abbey Matron — pays {W} to soak a hit.
pub fn abbey_matron() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ZERO,
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Abbey Matron",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            3,
        )
    }
}

/// Aysen Crusader — as big as the ranks behind it.
pub fn aysen_crusader() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(crate::card::DynamicPt::PermanentsControlledMatching {
            base_p: 2,
            base_t: 2,
            filter: Box::new(
                R::HasCreatureType(CreatureType::Soldier).or(R::HasCreatureType(CreatureType::Warrior)),
            ),
        }),
        ..creature(
            "Aysen Crusader",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Cemetery Gate — a wall black can't get past.
pub fn cemetery_gate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Protection(Color::Black)],
        ..creature("Cemetery Gate", cost(&[generic(2), b()]), vec![CreatureType::Wall], 0, 5)
    }
}

/// Chandler — three red mana melts an artifact creature.
pub fn chandler() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), r(), r()]),
            tap_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact.and(R::Creature)) },
            ..Default::default()
        }],
        ..creature(
            "Chandler",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Clockwork Gnomes — the artifact repair crew.
pub fn clockwork_gnomes() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Regenerate { what: target_filtered(R::Artifact.and(R::Creature)) },
            ..Default::default()
        }],
        ..artifact_creature(
            "Clockwork Gnomes",
            cost(&[generic(4)]),
            vec![CreatureType::Gnome],
            2,
            2,
        )
    }
}

/// Death Speakers — a one-drop black can't answer.
pub fn death_speakers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        ..creature(
            "Death Speakers",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Ebony Rhino — seven mana of trample.
pub fn ebony_rhino() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..artifact_creature("Ebony Rhino", cost(&[generic(7)]), vec![CreatureType::Rhino], 4, 5)
    }
}

/// Eron the Relentless — hasty, fragile, and hard to keep down.
pub fn eron_the_relentless() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), r(), r()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Eron the Relentless",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            5,
            2,
        )
    }
}

/// Faerie Noble — an anthem now and a bigger one on tap.
pub fn faerie_noble() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other Faerie creatures you control get +0/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Faerie)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
                power: 0,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Faerie)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Faerie Noble",
            cost(&[generic(2), g()]),
            vec![CreatureType::Faerie, CreatureType::Noble],
            1,
            2,
        )
    }
}

/// Folk of An-Havva — a one-drop that hits above its weight on defence.
pub fn folk_of_an_havva() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Folk of An-Havva", cost(&[g()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Grandmother Sengir — a repeatable point of shrink.
pub fn grandmother_sengir() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Grandmother Sengir",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Hungry Mist — six power that eats {G}{G} a turn.
pub fn hungry_mist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[g(), g()]) },
        }],
        ..creature(
            "Hungry Mist",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Elemental],
            6,
            2,
        )
    }
}

/// Ihsan's Shade — five power white can't block or burn.
pub fn ihsans_shade() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Protection(Color::White)],
        ..creature(
            "Ihsan's Shade",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Shade, CreatureType::Knight],
            5,
            5,
        )
    }
}

/// Joven — three red mana melts a noncreature artifact.
pub fn joven() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), r(), r()]),
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact.and(R::Creature.negate())),
            },
            ..Default::default()
        }],
        ..creature("Joven", cost(&[generic(3), r(), r()]), vec![CreatureType::Human, CreatureType::Rogue], 3, 3)
    }
}

/// Leaping Lizard — trades a point of toughness for the sky.
pub fn leaping_lizard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ZERO,
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Leaping Lizard", cost(&[generic(1), g(), g()]), vec![CreatureType::Lizard], 2, 3)
    }
}

/// Narwhal — first strike, and red can't stop it.
pub fn narwhal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Protection(Color::Red)],
        ..creature("Narwhal", cost(&[generic(2), u(), u()]), vec![CreatureType::Whale], 2, 2)
    }
}

/// Reef Pirates — every hit costs them the top of their library.
pub fn reef_pirates() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Reef Pirates",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Zombie, CreatureType::Pirate],
            2,
            2,
        )
    }
}

/// Root Spider — a blocker that strikes first.
pub fn root_spider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
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
        ..creature("Root Spider", cost(&[generic(3), g()]), vec![CreatureType::Spider], 2, 2)
    }
}

/// Sea Sprite — a flier red can't touch.
pub fn sea_sprite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        ..creature("Sea Sprite", cost(&[generic(1), u()]), vec![CreatureType::Faerie], 1, 1)
    }
}

/// Soraya the Falconer — the Bird lord.
pub fn soraya_the_falconer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Bird creatures get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Bird),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::HasCreatureType(CreatureType::Bird)),
                keyword: Keyword::Banding,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Soraya the Falconer",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human],
            2,
            2,
        )
    }
}

/// Veldrane of Sengir — trades power for a walk through the trees.
pub fn veldrane_of_sengir() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), b()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(-3),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Landwalk(LandType::Forest),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Veldrane of Sengir",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            5,
            5,
        )
    }
}

/// Wall of Kelp — a wall that grows more walls.
pub fn wall_of_kelp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Kelp".into(),
                    power: 0,
                    toughness: 1,
                    colors: vec![Color::Blue],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Plant, CreatureType::Wall],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Defender],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Wall of Kelp",
            cost(&[u(), u()]),
            vec![CreatureType::Plant, CreatureType::Wall],
            0,
            3,
        )
    }
}

/// Willow Faerie — a two-mana flier.
pub fn willow_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Willow Faerie", cost(&[generic(1), g()]), vec![CreatureType::Faerie], 1, 2)
    }
}

/// Dwarven Pony — carries a Dwarf over the mountains.
pub fn dwarven_pony() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::HasCreatureType(CreatureType::Dwarf)),
                keyword: Keyword::Landwalk(LandType::Mountain),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Dwarven Pony", cost(&[r()]), vec![CreatureType::Horse], 1, 1)
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Aliban's Tower — a blocker gets a lot bigger.
pub fn alibans_tower() -> CardDefinition {
    instant(
        "Aliban's Tower",
        cost(&[generic(1), r()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::IsBlocking)),
            power: Value::Const(3),
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Ambush — every blocker strikes first.
pub fn ambush() -> CardDefinition {
    instant(
        "Ambush",
        cost(&[generic(3), r()]),
        Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::IsBlocking)),
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        },
    )
}

/// An-Havva Inn — life for every green body on the board.
pub fn an_havva_inn() -> CardDefinition {
    sorcery(
        "An-Havva Inn",
        cost(&[generic(1), g(), g()]),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Sum(vec![
                Value::ONE,
                Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(R::Any)),
                    filter: R::Creature.and(R::HasColor(Color::Green)),
                },
            ]),
        },
    )
}

/// Dry Spell — a point across the whole table.
pub fn dry_spell() -> CardDefinition {
    sorcery(
        "Dry Spell",
        cost(&[generic(1), b()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::ONE,
            },
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
        ]),
    )
}

/// Evaporate — a point to everything white or blue.
pub fn evaporate() -> CardDefinition {
    sorcery(
        "Evaporate",
        cost(&[generic(2), r()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(
                R::Creature.and(R::HasColor(Color::White).or(R::HasColor(Color::Blue))),
            ),
            amount: Value::ONE,
        },
    )
}

/// Forget — two off the top of their hand, two back.
pub fn forget() -> CardDefinition {
    sorcery(
        "Forget",
        cost(&[u(), u()]),
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            },
            Effect::Draw {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Merchant Scroll — fetches the blue instant you need.
pub fn merchant_scroll() -> CardDefinition {
    sorcery(
        "Merchant Scroll",
        cost(&[generic(1), u()]),
        Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCardType(CardType::Instant).and(R::HasColor(Color::Blue)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Shrink — takes five power off an attacker.
pub fn shrink() -> CardDefinition {
    instant(
        "Shrink",
        cost(&[g()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-5),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Winter Sky — heads it burns the board, tails everyone draws.
pub fn winter_sky() -> CardDefinition {
    sorcery(
        "Winter Sky",
        cost(&[r()]),
        Effect::FlipCoinBy {
            flipper: PlayerRef::You,
            on_heads: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: Value::ONE,
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                },
            ])),
            on_tails: Box::new(Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
            }),
        },
    )
}

// ── Enchantments and artifacts ─────────────────────────────────────────────

/// Aysen Highway — white creatures walk past Plains.
pub fn aysen_highway() -> CardDefinition {
    enchantment(
        "Aysen Highway",
        cost(&[generic(3), w(), w(), w()]),
        vec![StaticAbility {
            description: "White creatures have plainswalk.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasColor(Color::White)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Landwalk(LandType::Plains)],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
    )
}

/// Feroz's Ban — every creature spell costs two more.
pub fn ferozs_ban() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creature spells cost {2} more to cast.",
            effect: StaticEffect::AdditionalCost { filter: R::Creature, amount: 2 },
        }],
        ..CardDefinition { name: "Feroz's Ban", cost: cost(&[generic(6)]), ..Default::default() }
    }
}

/// Irini Sengir — taxes green and white enchantments.
pub fn irini_sengir() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Green and white enchantment spells cost {2} more to cast.",
            effect: StaticEffect::AdditionalCost {
                filter: R::HasCardType(CardType::Enchantment)
                    .and(R::HasColor(Color::Green).or(R::HasColor(Color::White))),
                amount: 2,
            },
        }],
        ..creature(
            "Irini Sengir",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Dwarf],
            2,
            2,
        )
    }
}

/// Mystic Decree — grounds the whole board.
pub fn mystic_decree() -> CardDefinition {
    world_enchantment(
        "Mystic Decree",
        cost(&[generic(2), u(), u()]),
        vec![
            StaticAbility {
                description: "All creatures lose flying.",
                effect: StaticEffect::LoseKeyword {
                    applies_to: Selector::EachPermanent(R::Creature),
                    keyword: Keyword::Flying,
                },
            },
            StaticAbility {
                description: "All creatures lose islandwalk.",
                effect: StaticEffect::LoseKeyword {
                    applies_to: Selector::EachPermanent(R::Creature),
                    keyword: Keyword::Landwalk(LandType::Island),
                },
            },
        ],
    )
}

/// Primal Order — punishes a greedy mana base.
pub fn primal_order() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::CountMatching {
                    sel: Box::new(Selector::ControlledBy {
                        who: PlayerRef::ActivePlayer,
                        filter: R::Land,
                    }),
                    filter: R::IsBasicLand.negate(),
                },
            },
        }],
        ..enchantment("Primal Order", cost(&[generic(2), g(), g()]), vec![])
    }
}

/// Serra Aviary — the sky gets bigger.
pub fn serra_aviary() -> CardDefinition {
    world_enchantment(
        "Serra Aviary",
        cost(&[generic(3), w()]),
        vec![StaticAbility {
            description: "Creatures with flying get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
    )
}

/// Torture — a slow, repeatable shrink.
pub fn torture() -> CardDefinition {
    CardDefinition {
        name: "Torture",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::AddCounter {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}



/// Didgeridoo — puts a Minotaur straight onto the battlefield.
pub fn didgeridoo() -> CardDefinition {
    CardDefinition {
        name: "Didgeridoo",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::MayDo {
                description: "Put a Minotaur permanent card from your hand onto the battlefield"
                    .into(),
                body: Box::new(Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Minotaur),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Roterothopter — a cheap flier that can be pumped twice a turn.
pub fn roterothopter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            max_activations_per_turn: Some(2),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Roterothopter",
            cost(&[generic(1)]),
            vec![CreatureType::Thopter],
            0,
            2,
        )
    }
}

/// Aether Storm — nobody's creatures resolve until someone pays four life.
pub fn aether_storm() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 4,
            any_player: true,
            effect: Effect::DestroyNoRegen { what: Selector::This },
            ..Default::default()
        }],
        ..enchantment(
            "Aether Storm",
            cost(&[generic(3), u()]),
            vec![StaticAbility {
                description: "Creature spells can't be cast.",
                effect: StaticEffect::PlayersCantPlayMatching { filter: R::Creature },
            }],
        )
    }
}

// ── Second wave ────────────────────────────────────────────────────────────

/// An-Havva Constable — as tough as the green on the board.
pub fn an_havva_constable() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(crate::card::DynamicPt::PermanentsOnBattlefieldMatchingToughness {
            base_p: 2,
            base_t: 1,
            filter: Box::new(R::Creature.and(R::HasColor(Color::Green))),
        }),
        ..creature("An-Havva Constable", cost(&[generic(1), g(), g()]), vec![CreatureType::Human], 2, 1)
    }
}

/// Apocalypse Chime — unmakes Homelands.
pub fn apocalypse_chime() -> CardDefinition {
    CardDefinition {
        name: "Apocalypse Chime",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DestroyNoRegen {
                what: Selector::EachPermanent(
                    R::IsToken
                        .negate()
                        .and(R::OriginallyPrintedIn(crate::card::OriginalSet::Homelands)),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Beast Walkers — buys banding with green mana.
pub fn beast_walkers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Banding,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Beast Walkers",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Beast, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Labyrinth Minotaur — whatever it stops stays down a turn.
pub fn labyrinth_minotaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::SkipNextUntap { what: Selector::BlockedAttacker },
        }],
        ..creature("Labyrinth Minotaur", cost(&[generic(3), u()]), vec![CreatureType::Minotaur], 1, 4)
    }
}

/// Reveka, Wizard Savant — two damage, then a turn asleep.
pub fn reveka_wizard_savant() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                Effect::SkipNextUntap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Reveka, Wizard Savant",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Dwarf, CreatureType::Wizard],
            0,
            1,
        )
    }
}

/// Serra Paladin — a point of prevention, or vigilance for a friend.
pub fn serra_paladin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PreventNextDamage { target: target_any(), amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w(), w()]),
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Serra Paladin",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Sengir Bats — grows on every kill it helped with.
pub fn sengir_bats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Sengir Bats", cost(&[generic(1), b(), b()]), vec![CreatureType::Bat], 1, 2)
    }
}

/// Greater Werewolf — everything it meets in combat comes out smaller.
pub fn greater_werewolf() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::AddCounter {
                    what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                    kind: CounterType::MinusZeroMinusTwo,
                    amount: Value::ONE,
                }),
            },
        },
        TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::AddCounter {
                    what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                    kind: CounterType::MinusZeroMinusTwo,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature("Greater Werewolf", cost(&[generic(4), b()]), vec![CreatureType::Werewolf], 2, 4)
    }
}

/// Funeral March — the host's controller pays a second body when it goes.
pub fn funeral_march() -> CardDefinition {
    CardDefinition {
        name: "Funeral March",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::EnchantedBySource,
            ),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                count: Value::ONE,
                filter: R::Creature,
            },
        }],
        ..Default::default()
    }
}

/// Roots — pins a grounded creature to the floor.
pub fn roots() -> CardDefinition {
    CardDefinition {
        name: "Roots",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying).negate())),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap { what: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Spectral Bears — three power that sulks when nobody's playing black.
pub fn spectral_bears() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::Not(Box::new(Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::EachOpponent,
                    filter: R::HasColor(Color::Black).and(R::IsToken.negate()),
                }))),
            ),
            effect: Effect::SkipNextUntap { what: Selector::This },
        }],
        ..creature(
            "Spectral Bears",
            cost(&[generic(1), g()]),
            vec![CreatureType::Bear, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Rashka the Slayer — bigger when it stops black.
pub fn rashka_the_slayer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::BlockedAttacker,
                    filter: R::HasColor(Color::Black),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Rashka the Slayer",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Archer],
            3,
            3,
        )
    }
}

/// Serra Inquisitors — swells against black.
pub fn serra_inquisitors() -> CardDefinition {
    let pump = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(2),
        toughness: Value::ZERO,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::BlockedAttacker,
                        filter: R::HasColor(Color::Black),
                    },
                ),
                effect: pump.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::BlockingCreatures,
                        filter: R::HasColor(Color::Black),
                    }),
                effect: pump,
            },
        ],
        ..creature(
            "Serra Inquisitors",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Clockwork Steed — winds down as it fights, and winds back up at upkeep.
pub fn clockwork_steed() -> CardDefinition {
    clockwork("Clockwork Steed", CreatureType::Horse, Keyword::CantBeBlockedBy(Box::new(R::Artifact.and(R::Creature))))
}

/// Clockwork Swarm — the same engine, walled off instead.
pub fn clockwork_swarm() -> CardDefinition {
    clockwork(
        "Clockwork Swarm",
        CreatureType::Insect,
        Keyword::CantBeBlockedByCreatureType(CreatureType::Wall),
    )
}

/// The shared Clockwork body: four +1/+0 counters, one off at end of combat if
/// it fought, and an upkeep-only rewind capped at four.
fn clockwork(name: &'static str, kind: CreatureType, evasion: Keyword) -> CardDefinition {
    let shed = Effect::AtEndOfCombat {
        body: Box::new(Effect::RemoveCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusZero,
            amount: Value::ONE,
        }),
    };
    CardDefinition {
        keywords: vec![evasion],
        enters_with_counters: Some((CounterType::PlusOnePlusZero, Value::Const(4))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: shed.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: shed,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x()]),
            tap_cost: true,
            condition: Some(Predicate::CurrentStepIs(TurnStep::Upkeep)),
            effect: Effect::AddCounterCapped {
                what: Selector::This,
                kind: CounterType::PlusOnePlusZero,
                amount: Value::XFromCost,
                cap: Value::Const(4),
            },
            ..Default::default()
        }],
        ..artifact_creature(name, cost(&[generic(4)]), vec![kind], 0, 3)
    }
}

// ── Third wave ─────────────────────────────────────────────────────────────

/// "Activate only during combat."
fn during_combat() -> Predicate {
    Predicate::Any(
        [
            TurnStep::BeginCombat,
            TurnStep::DeclareAttackers,
            TurnStep::DeclareBlockers,
            TurnStep::FirstStrikeDamage,
            TurnStep::CombatDamage,
            TurnStep::EndCombat,
        ]
        .into_iter()
        .map(Predicate::CurrentStepIs)
        .collect(),
    )
}

/// Heart Wolf — lends a Dwarf its edge, and goes down with it.
pub fn heart_wolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(during_combat()),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::HasCreatureType(CreatureType::Dwarf)),
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::WhenTargetLeavesBattlefieldThisTurn {
                    what: Selector::Target(0),
                    body: Box::new(Effect::SacrificeSource),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Heart Wolf", cost(&[generic(3), r()]), vec![CreatureType::Wolf], 2, 2)
    }
}

/// Joven's Tools — only Walls get in the way.
pub fn jovens_tools() -> CardDefinition {
    CardDefinition {
        name: "Joven's Tools",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(
                    CreatureType::Wall,
                ))),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Willow Priestess — drops Faeries in for free and shields the green team.
pub fn willow_priestess() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::MayDo {
                    description: "Put a Faerie permanent card from your hand onto the battlefield"
                        .into(),
                    body: Box::new(Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::HasCreatureType(CreatureType::Faerie),
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    }),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::HasColor(Color::Green))),
                    keyword: Keyword::Protection(Color::Black),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Willow Priestess",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Faerie, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Dark Maze — a wall that can charge once, then vanishes.
pub fn dark_maze() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::AttackDespiteDefenderThisTurn { what: Selector::This },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Exile }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Dark Maze", cost(&[generic(4), u()]), vec![CreatureType::Wall], 4, 5)
    }
}

/// Samite Alchemist — a shield that costs the shielded creature a turn.
pub fn samite_alchemist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PreventNextDamage {
                    target: target_filtered(R::Creature.and(R::ControlledByYou)),
                    amount: Value::Const(4),
                },
                Effect::Tap { what: Selector::Target(0) },
                Effect::SkipNextUntap { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Samite Alchemist",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            0,
            2,
        )
    }
}

/// Joven's Ferrets — whatever stops it stays down a turn.
pub fn jovens_ferrets() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ZERO,
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::AtEndOfCombat {
                    body: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::BlockingCreatures },
                        Effect::SkipNextUntap { what: Selector::BlockingCreatures },
                    ])),
                },
            ]),
        }],
        ..creature("Joven's Ferrets", cost(&[g()]), vec![CreatureType::Ferret], 1, 1)
    }
}
