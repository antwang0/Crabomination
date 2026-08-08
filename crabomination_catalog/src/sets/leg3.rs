//! Legends (LEG) wave 4 — the Walls, the mana-battery cycle, the plain
//! legends and the set's one-line spells, enchantments and Auras. Tests in
//! `classic_sets/leg3`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Zone,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{gain_life, target, target_any, target_filtered, you},
};
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

/// A Legends legend body — `creature` plus the Legendary supertype.
fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, c, types, p, t) }
}

fn wall(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        ..creature(name, c, vec![CreatureType::Wall], p, t)
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

fn enchantment(name: &'static str, c: ManaCost, statics: Vec<StaticAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        static_abilities: statics,
        ..Default::default()
    }
}

/// A World enchantment body (CR 704.5k — the world rule keeps one in play).
fn world(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        supertypes: vec![Supertype::World],
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

/// An Aura body: `enchant` picks the host at cast time.
fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..Default::default()
    }
}

/// The Aura's live host.
fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// "Target instant or Aura spell that targets a permanent you control"
/// (Avoid Fate, Ring of Immortals). The engine's target-provenance filter also
/// catches a spell aimed at you.
fn instant_or_aura_spell() -> R {
    R::IsSpellOnStack
        .and(
            R::HasCardType(CardType::Instant)
                .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
        )
        .and(R::SpellTargetsControllerOrControlled)
}

// ── Walls ──────────────────────────────────────────────────────────────────

/// Wall of Heat — {2}{R} 2/6 Wall.
pub fn wall_of_heat() -> CardDefinition {
    wall("Wall of Heat", cost(&[generic(2), r()]), 2, 6)
}

/// Wall of Light — {2}{W} 1/5 Wall with protection from black.
pub fn wall_of_light() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Protection(Color::Black)],
        ..wall("Wall of Light", cost(&[generic(2), w()]), 1, 5)
    }
}

/// Wall of Opposition — {3}{R}{R} 0/6 Wall that pumps itself for {1}.
pub fn wall_of_opposition() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..wall("Wall of Opposition", cost(&[generic(3), r(), r()]), 0, 6)
    }
}

/// Wall of Vapor — {3}{U} 0/1 Wall the creatures it blocks can't hurt.
pub fn wall_of_vapor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to this creature by creatures \
                          it's blocking.",
            effect: StaticEffect::PreventAllDamageToThisFromBlocked,
        }],
        ..wall("Wall of Vapor", cost(&[generic(3), u()]), 0, 1)
    }
}

/// Wall of Shadows — {1}{B}{B} 0/1 Wall. (Its "can't be targeted by
/// Walls-only spells" clause has no engine analog — no such filter exists.)
pub fn wall_of_shadows() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to this creature by creatures \
                          it's blocking.",
            effect: StaticEffect::PreventAllDamageToThisFromBlocked,
        }],
        ..wall("Wall of Shadows", cost(&[generic(1), b(), b()]), 0, 1)
    }
}

/// Wall of Putrid Flesh — {2}{B} 2/4 Wall with protection from white.
pub fn wall_of_putrid_flesh() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Protection(Color::White)],
        ..wall("Wall of Putrid Flesh", cost(&[generic(2), b()]), 2, 4)
    }
}

/// Wall of Tombstones — {1}{B} 0/1 Wall whose base toughness climbs with your
/// graveyard each upkeep.
pub fn wall_of_tombstones() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::SetBasePT {
                what: Selector::This,
                power: Value::ZERO,
                toughness: Value::Sum(vec![
                    Value::ONE,
                    Value::CountOf(Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Creature,
                    })),
                ]),
                duration: Duration::Permanent,
            },
        }],
        ..wall("Wall of Tombstones", cost(&[generic(1), b()]), 0, 1)
    }
}

// ── Plain creatures ────────────────────────────────────────────────────────

/// Thunder Spirit — {1}{W}{W} 2/2 with flying and first strike.
pub fn thunder_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..creature(
            "Thunder Spirit",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Elemental, CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Wolverine Pack — {2}{G}{G} 2/4 with rampage 2.
pub fn wolverine_pack() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rampage(2)],
        ..creature(
            "Wolverine Pack",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Wolverine],
            2,
            4,
        )
    }
}

/// Righteous Avengers — {4}{W} 3/1 with plainswalk.
pub fn righteous_avengers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Plains)],
        ..creature(
            "Righteous Avengers",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            1,
        )
    }
}

/// Beasts of Bogardan — {4}{R} 3/3 with protection from red, bigger while an
/// opponent has a nontoken white permanent.
pub fn beasts_of_bogardan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 as long as an opponent controls a nontoken \
                          white permanent.",
            effect: StaticEffect::PumpSelfIf {
                power: 1,
                toughness: 1,
                keywords: vec![],
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::ControlledByOpponent.and(R::HasColor(Color::White)).and(R::NotToken),
                )),
            },
        }],
        ..creature("Beasts of Bogardan", cost(&[generic(4), r()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Ivory Guardians — {4}{W}{W} 3/3 with protection from red; the whole
/// name-sharing crew grows against a red opponent.
pub fn ivory_guardians() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        static_abilities: vec![StaticAbility {
            description: "Creatures named Ivory Guardians get +1/+1 as long as an opponent \
                          controls a nontoken red permanent.",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::HasName("Ivory Guardians".into()),
                power: 1,
                toughness: 1,
                keywords: vec![],
                all_players: true,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::ControlledByOpponent.and(R::HasColor(Color::Red)).and(R::NotToken),
                )),
            },
        }],
        ..creature(
            "Ivory Guardians",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Giant, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Elder Land Wurm — {4}{W}{W}{W} 5/5 defender/trample that sheds defender
/// the moment it blocks.
pub fn elder_land_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::LoseKeyword {
                what: Selector::This,
                keyword: Keyword::Defender,
                duration: Duration::Permanent,
            },
        }],
        ..creature(
            "Elder Land Wurm",
            cost(&[generic(4), w(), w(), w()]),
            vec![CreatureType::Dragon, CreatureType::Wurm],
            5,
            5,
        )
    }
}

/// Spinal Villain — {2}{R} 1/2 that taps to kill a blue creature.
pub fn spinal_villain() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::HasColor(Color::Blue))) },
            ..Default::default()
        }],
        ..creature("Spinal Villain", cost(&[generic(2), r()]), vec![CreatureType::Beast], 1, 2)
    }
}

// ── The plain legends ──────────────────────────────────────────────────────

/// Sir Shandlar of Eberyn — {4}{G}{W} 4/7.
pub fn sir_shandlar_of_eberyn() -> CardDefinition {
    legend(
        "Sir Shandlar of Eberyn",
        cost(&[generic(4), g(), w()]),
        vec![CreatureType::Human, CreatureType::Knight],
        4,
        7,
    )
}

/// Sivitri Scarzam — {5}{U}{B} 6/4.
pub fn sivitri_scarzam() -> CardDefinition {
    legend("Sivitri Scarzam", cost(&[generic(5), u(), b()]), vec![CreatureType::Human], 6, 4)
}

/// The Lady of the Mountain — {4}{R}{G} 5/5.
pub fn the_lady_of_the_mountain() -> CardDefinition {
    legend(
        "The Lady of the Mountain",
        cost(&[generic(4), r(), g()]),
        vec![CreatureType::Giant],
        5,
        5,
    )
}

/// Tobias Andrion — {3}{W}{U} 4/4.
pub fn tobias_andrion() -> CardDefinition {
    legend(
        "Tobias Andrion",
        cost(&[generic(3), w(), u()]),
        vec![CreatureType::Human, CreatureType::Advisor],
        4,
        4,
    )
}

/// Torsten Von Ursus — {3}{G}{G}{W} 5/5.
pub fn torsten_von_ursus() -> CardDefinition {
    legend(
        "Torsten Von Ursus",
        cost(&[generic(3), g(), g(), w()]),
        vec![CreatureType::Human, CreatureType::Soldier],
        5,
        5,
    )
}

/// Riven Turnbull — {5}{U}{B} 5/7 that taps for {B}.
pub fn riven_turnbull() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black]),
            },
            ..Default::default()
        }],
        ..legend(
            "Riven Turnbull",
            cost(&[generic(5), u(), b()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            5,
            7,
        )
    }
}

/// Sunastian Falconer — {3}{R}{G} 4/4 that taps for {C}{C}.
pub fn sunastian_falconer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..legend(
            "Sunastian Falconer",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            4,
            4,
        )
    }
}

/// Xira Arien — {B}{R}{G} 1/2 flier who sells cards.
pub fn xira_arien() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[b(), r(), g()]),
            effect: Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
            ..Default::default()
        }],
        ..legend(
            "Xira Arien",
            cost(&[b(), r(), g()]),
            vec![CreatureType::Insect, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Tuknir Deathlock — {R}{R}{G}{G} 2/2 flier with a repeatable pump.
pub fn tuknir_deathlock() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r(), g()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Tuknir Deathlock",
            cost(&[r(), r(), g(), g()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Tor Wauki — {2}{B}{B}{R} 3/3 archer who shoots combatants.
pub fn tor_wauki() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..legend(
            "Tor Wauki",
            cost(&[generic(2), b(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Archer],
            3,
            3,
        )
    }
}

/// Adun Oakenshield — {B}{R}{G} 1/2 who buys creatures back.
pub fn adun_oakenshield() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[b(), r(), g()]),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..legend(
            "Adun Oakenshield",
            cost(&[b(), r(), g()]),
            vec![CreatureType::Human, CreatureType::Knight],
            1,
            2,
        )
    }
}

/// Lady Evangela — {W}{U}{B} 1/2 who blanks one attacker's swing.
pub fn lady_evangela() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w(), b()]),
            effect: Effect::PreventCombatDamageByTargetThisTurn {
                target: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..legend(
            "Lady Evangela",
            cost(&[w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Angus Mackenzie — {G}{W}{U} 2/2 whose tap is a repeatable fog.
pub fn angus_mackenzie() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[g(), w(), u()]),
            effect: Effect::PreventAllCombatDamageThisTurn,
            ..Default::default()
        }],
        ..legend(
            "Angus Mackenzie",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Sol'kanar the Swamp King — {2}{U}{B}{R} 5/5 swampwalker who taxes every
/// black spell cast.
pub fn solkanar_the_swamp_king() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Black),
                },
            ),
            effect: gain_life(1),
        }],
        ..legend(
            "Sol'kanar the Swamp King",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Demon],
            5,
            5,
        )
    }
}

/// Ur-Drago — {3}{U}{U}{B}{B} 4/4 first striker who blanks swampwalk.
pub fn ur_drago() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Creatures with swampwalk can be blocked as though they didn't have \
                          swampwalk.",
            effect: StaticEffect::LandwalkIgnored(LandType::Swamp),
        }],
        ..legend(
            "Ur-Drago",
            cost(&[generic(3), u(), u(), b(), b()]),
            vec![CreatureType::Elemental],
            4,
            4,
        )
    }
}

// ── The mana-battery cycle ─────────────────────────────────────────────────

/// The Legends mana batteries: bank charge counters, then cash them all in
/// for one mana plus one per counter removed.
fn mana_battery(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_x: Some(CounterType::Charge),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(
                        color,
                        Value::Sum(vec![Value::ONE, Value::XFromCost]),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// White Mana Battery.
pub fn white_mana_battery() -> CardDefinition {
    mana_battery("White Mana Battery", Color::White)
}

/// Blue Mana Battery.
pub fn blue_mana_battery() -> CardDefinition {
    mana_battery("Blue Mana Battery", Color::Blue)
}

/// Black Mana Battery.
pub fn black_mana_battery() -> CardDefinition {
    mana_battery("Black Mana Battery", Color::Black)
}

/// Red Mana Battery.
pub fn red_mana_battery() -> CardDefinition {
    mana_battery("Red Mana Battery", Color::Red)
}

/// Green Mana Battery.
pub fn green_mana_battery() -> CardDefinition {
    mana_battery("Green Mana Battery", Color::Green)
}

// ── Other artifacts ────────────────────────────────────────────────────────

/// Horn of Deafening — {4} artifact that silences one attacker per turn.
pub fn horn_of_deafening() -> CardDefinition {
    CardDefinition {
        name: "Horn of Deafening",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PreventCombatDamageByTargetThisTurn {
                target: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Serpent Generator — {6} artifact that mints poisonous Snakes.
pub fn serpent_generator() -> CardDefinition {
    CardDefinition {
        name: "Serpent Generator",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Snake".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Snake],
                        ..Default::default()
                    },
                    triggered_abilities: vec![TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::DealsCombatDamageToPlayer,
                            EventScope::SelfSource,
                        ),
                        effect: Effect::AddPoison {
                            who: Selector::Player(PlayerRef::TriggerEventPlayer),
                            amount: Value::ONE,
                        },
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Life Chisel — {4} artifact that trades a creature for its toughness in
/// life, at upkeep only.
pub fn life_chisel() -> CardDefinition {
    CardDefinition {
        name: "Life Chisel",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            condition: Some(Predicate::CurrentStepIs(crate::game::types::TurnStep::Upkeep)),
            effect: Effect::GainLife { who: you(), amount: Value::SacrificedToughness },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Syphon Soul — {2}{B}. Two to each opponent, back as life.
pub fn syphon_soul() -> CardDefinition {
    sorcery(
        "Syphon Soul",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            Effect::GainLife { who: you(), amount: Value::DamageDealtThisResolution },
        ]),
    )
}

/// Storm Seeker — {3}{G}. Damage equal to the victim's hand.
pub fn storm_seeker() -> CardDefinition {
    instant(
        "Storm Seeker",
        cost(&[generic(3), g()]),
        Effect::DealDamage {
            to: Selector::Player(PlayerRef::Target(0)),
            amount: Value::HandSizeOf(PlayerRef::Target(0)),
        },
    )
}

/// Untamed Wilds — {2}{G}. Fetch a basic onto the battlefield.
pub fn untamed_wilds() -> CardDefinition {
    sorcery(
        "Untamed Wilds",
        cost(&[generic(2), g()]),
        Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Teleport — {U}{U}{U}. Cast during declare attackers; that creature is
/// through.
pub fn teleport() -> CardDefinition {
    CardDefinition {
        cast_only_during_combat: true,
        ..instant(
            "Teleport",
            cost(&[u(), u(), u()]),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Reset — {U}{U}. Untap your lands on their turn.
pub fn reset() -> CardDefinition {
    instant(
        "Reset",
        cost(&[u(), u()]),
        Effect::Untap {
            what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            up_to: None,
        },
    )
}

/// Rust — {G}. Counter an artifact's activated ability.
pub fn rust() -> CardDefinition {
    instant(
        "Rust",
        cost(&[g()]),
        Effect::CounterAbility { what: target_filtered(R::Artifact) },
    )
}

/// Psychic Purge — {U}. A one-point ping. (Its discard punisher needs a
/// "made you discard this card" trigger the engine has no hook for.)
pub fn psychic_purge() -> CardDefinition {
    sorcery(
        "Psychic Purge",
        cost(&[u()]),
        Effect::DealDamage { to: target_any(), amount: Value::ONE },
    )
}

// ── Enchantments ───────────────────────────────────────────────────────────

/// Presence of the Master — {3}{W}. Nobody resolves an enchantment.
pub fn presence_of_the_master() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                },
            ),
            effect: Effect::CounterSpell { what: Selector::TriggerSource },
        }],
        ..enchantment("Presence of the Master", cost(&[generic(3), w()]), vec![])
    }
}

/// Undertow — {2}{U}. Islandwalk stops working.
pub fn undertow() -> CardDefinition {
    enchantment(
        "Undertow",
        cost(&[generic(2), u()]),
        vec![StaticAbility {
            description: "Creatures with islandwalk can be blocked as though they didn't have \
                          islandwalk.",
            effect: StaticEffect::LandwalkIgnored(LandType::Island),
        }],
    )
}

/// Revelation — {G} World Enchantment. Everyone shows their hand.
pub fn revelation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players play with their hands revealed.",
            effect: StaticEffect::OpponentsPlayWithHandsRevealed,
        }],
        ..world("Revelation", cost(&[g()]))
    }
}

/// Caverns of Despair — {2}{R}{R} World Enchantment. Two on, two back.
pub fn caverns_of_despair() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "No more than two creatures can attack each combat.",
                effect: StaticEffect::MaxAttackersPerCombat(2),
            },
            StaticAbility {
                description: "No more than two creatures can block each combat.",
                effect: StaticEffect::MaxBlockersPerCombat(2),
            },
        ],
        ..world("Caverns of Despair", cost(&[generic(2), r(), r()]))
    }
}

/// Living Plane — {2}{G}{G} World Enchantment. Every land is a 1/1.
pub fn living_plane() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All lands are 1/1 creatures that are still lands.",
            effect: StaticEffect::MatchingLandsAreCreatures {
                filter: R::Land,
                power: 1,
                toughness: 1,
                keywords: vec![],
                creature_types: vec![],
                colors: vec![],
            },
        }],
        ..world("Living Plane", cost(&[generic(2), g(), g()]))
    }
}

/// In the Eye of Chaos — {2}{U} World Enchantment. Instants cost their own
/// mana value again.
pub fn in_the_eye_of_chaos() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCardType(CardType::Instant) },
            ),
            effect: Effect::CounterUnlessPaid {
                what: Selector::TriggerSource,
                mana_cost: ManaCost::default(),
                exile: false,
                extra_generic: Some(Value::ManaValueOf(Box::new(Selector::TriggerSource))),
            },
        }],
        ..world("In the Eye of Chaos", cost(&[generic(2), u()]))
    }
}

// ── Auras ──────────────────────────────────────────────────────────────────

/// Spirit Link — {W} Aura. Its host's damage feeds you.
pub fn spirit_link() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::EnchantedBySource),
            effect: Effect::GainLife { who: you(), amount: Value::TriggerEventAmount },
        }],
        ..aura("Spirit Link", cost(&[w()]), R::Creature)
    }
}

/// Spirit Shackle — {B}{B} Aura. Tapping the host shrinks it for good.
pub fn spirit_shackle() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::MinusZeroMinusTwo,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        }),
        ..aura("Spirit Shackle", cost(&[b(), b()]), R::Creature)
    }
}

/// Seeker — {2}{W}{W} Aura. Only artifacts and white creatures get in the way.
pub fn seeker() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature can't be blocked except by artifact creatures \
                          and/or white creatures.",
            effect: StaticEffect::GrantKeyword {
                applies_to: host(),
                keyword: Keyword::CantBeBlockedExceptBy(Box::new(
                    R::Artifact.or(R::HasColor(Color::White)),
                )),
            },
        }],
        ..aura("Seeker", cost(&[generic(2), w(), w()]), R::Creature)
    }
}

/// Spectral Cloak — {U}{U} Aura. Untapped, the host is untouchable.
pub fn spectral_cloak() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has shroud as long as it's untapped.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: host(), filter: R::Untapped },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: host(),
                    keyword: Keyword::Shroud,
                }),
            },
        }],
        ..aura("Spectral Cloak", cost(&[u(), u()]), R::Creature)
    }
}

/// Demonic Torment — {2}{B} Aura. The host neither attacks nor connects.
pub fn demonic_torment() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature can't attack.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: host(),
                    keyword: Keyword::CantAttack,
                },
            },
            StaticAbility {
                description: "Prevent all combat damage that would be dealt by enchanted \
                              creature.",
                effect: StaticEffect::PreventAllDamageByEnchanted,
            },
        ],
        ..aura("Demonic Torment", cost(&[generic(2), b()]), R::Creature)
    }
}

/// Blight — {B}{B} Aura on a land. Tap it and it dies.
pub fn blight() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::Destroy { what: Selector::This },
            }],
            ..Default::default()
        }),
        ..aura("Blight", cost(&[b(), b()]), R::Land)
    }
}

/// Ring of Immortals — {5} artifact that counters a permanent-protecting
/// spell for {3}.
pub fn ring_of_immortals() -> CardDefinition {
    CardDefinition {
        name: "Ring of Immortals",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::CounterSpell {
                what: target_filtered(
                    instant_or_aura_spell(),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Avoid Fate — {G}. The instant-speed half of Ring of Immortals.
pub fn avoid_fate() -> CardDefinition {
    instant(
        "Avoid Fate",
        cost(&[g()]),
        Effect::CounterSpell {
            what: target_filtered(
                instant_or_aura_spell(),
            ),
        },
    )
}

/// Subdue — {G}. A creature stops hitting and starts absorbing.
pub fn subdue() -> CardDefinition {
    instant(
        "Subdue",
        cost(&[g()]),
        Effect::Seq(vec![
            Effect::PreventCombatDamageByTargetThisTurn { target: target() },
            Effect::PumpPT {
                what: target(),
                power: Value::ZERO,
                toughness: Value::ManaValueOf(Box::new(target())),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}
