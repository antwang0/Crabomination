//! Legends (LEG) wave 3 — the landwalk-hosers, the "becomes [color]" cycle,
//! the legend bodies and the one-line utility cards. Tests in
//! `classic_sets/leg2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value,
    shortcut::{etb, target_any, target_filtered},
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

// ── The landwalk hosers (CR 509.1b) ────────────────────────────────────────

fn landwalk_hoser(name: &'static str, c: ManaCost, lt: LandType, word: &'static str) -> CardDefinition {
    enchantment(
        name,
        c,
        vec![StaticAbility {
            description: word,
            effect: StaticEffect::LandwalkIgnored(lt),
        }],
    )
}

/// Great Wall — plainswalk stops working.
pub fn great_wall() -> CardDefinition {
    landwalk_hoser(
        "Great Wall",
        cost(&[generic(2), w()]),
        LandType::Plains,
        "Creatures with plainswalk can be blocked as though they didn't have plainswalk.",
    )
}

/// Deadfall — forestwalk stops working.
pub fn deadfall() -> CardDefinition {
    landwalk_hoser(
        "Deadfall",
        cost(&[generic(2), g()]),
        LandType::Forest,
        "Creatures with forestwalk can be blocked as though they didn't have forestwalk.",
    )
}

/// Quagmire — swampwalk stops working.
pub fn quagmire() -> CardDefinition {
    landwalk_hoser(
        "Quagmire",
        cost(&[generic(2), b()]),
        LandType::Swamp,
        "Creatures with swampwalk can be blocked as though they didn't have swampwalk.",
    )
}

/// Crevasse — mountainwalk stops working.
pub fn crevasse() -> CardDefinition {
    landwalk_hoser(
        "Crevasse",
        cost(&[generic(2), r()]),
        LandType::Mountain,
        "Creatures with mountainwalk can be blocked as though they didn't have mountainwalk.",
    )
}

/// Gosta Dirk — a first-striking legend who hoses islandwalk.
pub fn gosta_dirk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Creatures with islandwalk can be blocked as though they didn't have \
                          islandwalk.",
            effect: StaticEffect::LandwalkIgnored(LandType::Island),
        }],
        ..legend(
            "Gosta Dirk",
            cost(&[generic(3), w(), w(), u(), u()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Lord Magnus — hoses plainswalk and forestwalk at once.
pub fn lord_magnus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures with plainswalk can be blocked as though they didn't \
                              have plainswalk.",
                effect: StaticEffect::LandwalkIgnored(LandType::Plains),
            },
            StaticAbility {
                description: "Creatures with forestwalk can be blocked as though they didn't \
                              have forestwalk.",
                effect: StaticEffect::LandwalkIgnored(LandType::Forest),
            },
        ],
        ..legend(
            "Lord Magnus",
            cost(&[generic(3), g(), w(), w()]),
            vec![CreatureType::Human, CreatureType::Druid],
            4,
            3,
        )
    }
}

/// Hammerheim — red mana, or strip a creature's landwalk.
pub fn hammerheim() -> CardDefinition {
    CardDefinition {
        name: "Hammerheim",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Red]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::LoseAllLandwalk {
                    what: target_filtered(R::Creature),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Indestructible Aura — a one-creature fog.
pub fn indestructible_aura() -> CardDefinition {
    instant(
        "Indestructible Aura",
        cost(&[w()]),
        Effect::PreventAllDamageThisTurn {
            target: target_filtered(R::Creature),
            redirect_to: None,
        },
    )
}

/// Acid Rain — a one-sided Forest sweeper.
pub fn acid_rain() -> CardDefinition {
    sorcery(
        "Acid Rain",
        cost(&[generic(3), u()]),
        Effect::Destroy { what: Selector::EachPermanent(R::HasLandType(LandType::Forest)) },
    )
}

/// Cleanse — a black-creature sweeper.
pub fn cleanse() -> CardDefinition {
    sorcery(
        "Cleanse",
        cost(&[generic(2), w(), w()]),
        Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Black))),
        },
    )
}

/// Active Volcano — kill a blue permanent or bounce an Island.
pub fn active_volcano() -> CardDefinition {
    instant(
        "Active Volcano",
        cost(&[r()]),
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::HasColor(Color::Blue)) },
            Effect::Move {
                what: target_filtered(R::HasLandType(LandType::Island)),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
    )
}

/// Flash Flood — kill a red permanent or bounce a Mountain.
pub fn flash_flood() -> CardDefinition {
    instant(
        "Flash Flood",
        cost(&[u()]),
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::HasColor(Color::Red)) },
            Effect::Move {
                what: target_filtered(R::HasLandType(LandType::Mountain)),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
    )
}

/// Jovial Evil — burn scaled off the target's white creatures.
pub fn jovial_evil() -> CardDefinition {
    sorcery(
        "Jovial Evil",
        cost(&[generic(2), b()]),
        Effect::DealDamage {
            to: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Times(
                Box::new(Value::PermanentCountControlledByMatching(
                    PlayerRef::Target(0),
                    R::Creature.and(R::HasColor(Color::White)),
                )),
                Box::new(Value::Const(2)),
            ),
        },
    )
}

// ── Enchantments ───────────────────────────────────────────────────────────

/// Moat — the ground can't get through.
pub fn moat() -> CardDefinition {
    enchantment(
        "Moat",
        cost(&[generic(2), w(), w()]),
        vec![StaticAbility {
            description: "Creatures without flying can't attack.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                keyword: Keyword::CantAttack,
            },
        }],
    )
}

/// Gravity Sphere — a world enchantment that grounds everything.
pub fn gravity_sphere() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::World],
        ..enchantment(
            "Gravity Sphere",
            cost(&[generic(2), r()]),
            vec![StaticAbility {
                description: "All creatures lose flying.",
                effect: StaticEffect::LoseKeyword {
                    applies_to: Selector::EachPermanent(R::Creature),
                    keyword: Keyword::Flying,
                },
            }],
        )
    }
}

/// Greed — life into cards.
pub fn greed() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 2,
            effect: crate::effect::shortcut::draw(1),
            ..Default::default()
        }],
        ..enchantment("Greed", cost(&[generic(3), b()]), vec![])
    }
}

/// Planar Gate — creature spells come down cheaper.
pub fn planar_gate() -> CardDefinition {
    CardDefinition {
        name: "Planar Gate",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast cost {2} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Creature, amount: 2 },
        }],
        ..Default::default()
    }
}

/// Mana Matrix — instants and enchantments come down cheaper.
pub fn mana_matrix() -> CardDefinition {
    CardDefinition {
        name: "Mana Matrix",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Instant and enchantment spells you cast cost {2} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Enchantment)),
                amount: 2,
            },
        }],
        ..Default::default()
    }
}

/// Arena of the Ancients — legends stay tapped.
pub fn arena_of_the_ancients() -> CardDefinition {
    CardDefinition {
        name: "Arena of the Ancients",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Legendary creatures don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                ),
            },
        }],
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::EachPermanent(
                R::Creature.and(R::HasSupertype(Supertype::Legendary)),
            ),
        })],
        ..Default::default()
    }
}

/// Divine Transformation — the biggest of the old pump Auras.
pub fn divine_transformation() -> CardDefinition {
    CardDefinition {
        name: "Divine Transformation",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 3, ..Default::default() }),
        ..Default::default()
    }
}

/// Jacques le Vert — a green toughness anthem on a body.
pub fn jacques_le_vert() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Green creatures you control get +0/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByYou).and(R::HasColor(Color::Green)),
                power: 0,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..legend(
            "Jacques le Vert",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            2,
        )
    }
}

/// Fortified Area — Walls get a push and learn to band.
pub fn fortified_area() -> CardDefinition {
    enchantment(
        "Fortified Area",
        cost(&[generic(1), w(), w()]),
        vec![StaticAbility {
            description: "Wall creatures you control get +1/+0 and have banding.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature
                    .and(R::ControlledByYou)
                    .and(R::HasCreatureType(CreatureType::Wall)),
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Banding],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
    )
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Dakkon Blackblade — as big as your mana base.
pub fn dakkon_blackblade() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching {
            base_p: 0,
            base_t: 0,
            filter: Box::new(R::Land),
        }),
        ..legend(
            "Dakkon Blackblade",
            cost(&[generic(2), w(), u(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            0,
            0,
        )
    }
}

/// Fallen Angel — eats your board for a bigger swing.
pub fn fallen_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Fallen Angel", cost(&[generic(3), b(), b()]), vec![CreatureType::Angel], 3, 3)
    }
}

/// Killer Bees — a firebreathing flier that grows both ways.
pub fn killer_bees() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Killer Bees", cost(&[generic(1), g(), g()]), vec![CreatureType::Insect], 0, 1)
    }
}

/// Pavel Maliki — a legend that firebreathes for two colors.
pub fn pavel_maliki() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Pavel Maliki",
            cost(&[generic(4), b(), r()]),
            vec![CreatureType::Human],
            5,
            3,
        )
    }
}

/// Princess Lucrezia — a legend that taps for blue.
pub fn princess_lucrezia() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Blue]),
            },
            ..Default::default()
        }],
        ..legend(
            "Princess Lucrezia",
            cost(&[generic(3), u(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            5,
            4,
        )
    }
}

/// Cyclopean Mummy — it doesn't stay in the graveyard.
pub fn cyclopean_mummy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move { what: Selector::This, to: crate::effect::ZoneDest::Exile },
        }],
        ..creature("Cyclopean Mummy", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 2, 1)
    }
}

/// Crimson Manticore — a flier that pings the combat.
pub fn crimson_manticore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r()]),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Crimson Manticore",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Manticore],
            2,
            2,
        )
    }
}

/// Lady Caleria — a bigger combat sniper.
pub fn lady_caleria() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..legend(
            "Lady Caleria",
            cost(&[generic(3), g(), g(), w(), w()]),
            vec![CreatureType::Elf, CreatureType::Archer],
            3,
            6,
        )
    }
}

/// Pradesh Gypsies — a repeatable power shave.
pub fn pradesh_gypsies() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Pradesh Gypsies",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            1,
            1,
        )
    }
}

/// Ragnar — a Bant legend with a regeneration button.
pub fn ragnar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[g(), w(), u()]),
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..legend(
            "Ragnar",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Kei Takahashi — a repeatable damage shield.
pub fn kei_takahashi() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..legend(
            "Kei Takahashi",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Pixie Queen — hands out flying.
pub fn pixie_queen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[g(), g(), g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Pixie Queen", cost(&[generic(2), g(), g()]), vec![CreatureType::Faerie], 1, 1)
    }
}

/// Gwendlyn Di Corci — a repeatable random discard.
pub fn gwendlyn_di_corci() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: true,
            },
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            ..Default::default()
        }],
        ..legend(
            "Gwendlyn Di Corci",
            cost(&[u(), b(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            5,
        )
    }
}

/// Boris Devilboon — a Minor Demon factory.
pub fn boris_devilboon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), b(), r()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Minor Demon".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Black, Color::Red],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Demon],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..legend(
            "Boris Devilboon",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Psionic Entity — it can ping anything, at a price.
pub fn psionic_entity() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                Effect::DealDamage { to: Selector::This, amount: Value::Const(3) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Psionic Entity",
            cost(&[generic(4), u()]),
            vec![CreatureType::Illusion],
            2,
            2,
        )
    }
}

/// Hyperion Blacksmith — jams an opponent's artifacts.
pub fn hyperion_blacksmith() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TapOrUntap {
                what: target_filtered(R::Artifact.and(R::ControlledByOpponent)),
            },
            ..Default::default()
        }],
        ..creature(
            "Hyperion Blacksmith",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// Ramses Overdark — assassinates anything wearing an Aura.
pub fn ramses_overdark() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::IsEnchanted)),
            },
            ..Default::default()
        }],
        ..legend(
            "Ramses Overdark",
            cost(&[generic(2), u(), u(), b(), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            4,
            3,
        )
    }
}

/// Elven Riders — only Walls and fliers get in the way.
pub fn elven_riders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(
            R::HasCreatureType(CreatureType::Wall).or(R::HasKeyword(Keyword::Flying)),
        ))],
        ..creature("Elven Riders", cost(&[generic(3), g(), g()]), vec![CreatureType::Elf], 3, 3)
    }
}

/// Osai Vultures — it fattens on the graveyard.
pub fn osai_vultures() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Carrion,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Carrion, 2)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Osai Vultures", cost(&[generic(1), w()]), vec![CreatureType::Bird], 1, 1)
    }
}

// ── Vanilla legends ────────────────────────────────────────────────────────

/// Barktooth Warbeard — a vanilla 6/5 legend.
pub fn barktooth_warbeard() -> CardDefinition {
    legend(
        "Barktooth Warbeard",
        cost(&[generic(4), b(), r(), r()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        6,
        5,
    )
}

/// Jasmine Boreal — a vanilla 4/5 legend.
pub fn jasmine_boreal() -> CardDefinition {
    legend(
        "Jasmine Boreal",
        cost(&[generic(3), g(), w()]),
        vec![CreatureType::Human],
        4,
        5,
    )
}

/// Jedit Ojanen — a vanilla 5/5 legend.
pub fn jedit_ojanen() -> CardDefinition {
    legend(
        "Jedit Ojanen",
        cost(&[generic(4), w(), w(), u()]),
        vec![CreatureType::Cat, CreatureType::Warrior],
        5,
        5,
    )
}

/// Jerrard of the Closed Fist — a vanilla 6/5 legend.
pub fn jerrard_of_the_closed_fist() -> CardDefinition {
    legend(
        "Jerrard of the Closed Fist",
        cost(&[generic(3), r(), g(), g()]),
        vec![CreatureType::Human, CreatureType::Knight],
        6,
        5,
    )
}

/// Kasimir the Lone Wolf — a vanilla 5/3 legend.
pub fn kasimir_the_lone_wolf() -> CardDefinition {
    legend(
        "Kasimir the Lone Wolf",
        cost(&[generic(4), w(), u()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        5,
        3,
    )
}

/// Lady Orca — a vanilla 7/4 legend.
pub fn lady_orca() -> CardDefinition {
    legend("Lady Orca", cost(&[generic(5), b(), r()]), vec![CreatureType::Demon], 7, 4)
}

/// Ramirez DePietro — a first-striking legend.
pub fn ramirez_depietro() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..legend(
            "Ramirez DePietro",
            cost(&[generic(3), u(), b(), b()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            4,
            3,
        )
    }
}

/// Keepers of the Faith — a vanilla 2/3.
pub fn keepers_of_the_faith() -> CardDefinition {
    creature(
        "Keepers of the Faith",
        cost(&[generic(1), w(), w()]),
        vec![CreatureType::Human, CreatureType::Cleric],
        2,
        3,
    )
}

/// Moss Monster — a vanilla 3/6.
pub fn moss_monster() -> CardDefinition {
    creature(
        "Moss Monster",
        cost(&[generic(3), g(), g()]),
        vec![CreatureType::Elemental],
        3,
        6,
    )
}

/// Raging Bull — a vanilla 2/2.
pub fn raging_bull() -> CardDefinition {
    creature("Raging Bull", cost(&[generic(2), r()]), vec![CreatureType::Ox], 2, 2)
}

/// Lost Soul — a swampwalking 2/1.
pub fn lost_soul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..creature(
            "Lost Soul",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Spirit, CreatureType::Minion],
            2,
            1,
        )
    }
}

/// Hunding Gjornersen — rampage 1 on a 5/4.
pub fn hunding_gjornersen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rampage(1)],
        ..legend(
            "Hunding Gjornersen",
            cost(&[generic(3), w(), u(), u()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            5,
            4,
        )
    }
}

/// Marhault Elsdragon — rampage 1 on a 4/6.
pub fn marhault_elsdragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rampage(1)],
        ..legend(
            "Marhault Elsdragon",
            cost(&[generic(3), r(), r(), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            4,
            6,
        )
    }
}

/// Aerathi Berserker — rampage 3 on a 2/4.
pub fn aerathi_berserker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rampage(3)],
        ..creature(
            "Aerathi Berserker",
            cost(&[generic(2), r(), r(), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            2,
            4,
        )
    }
}
