//! Commander: the 2026-10-16 (`claude/modern_decks`, third leg) rescan's
//! short-text gaps — Black Widow's vanilla bodies, X-23's two Marvel heroes,
//! The Howling Abomination. Tests in `tests/recent_b/cmdr_oct16_e.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, ModesWiden, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{deal, each_opponent, etb, gain_life, on_attack, on_dies, on_you_attack, target_filtered, unearth};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use crate::sets::{tap_add_any_color, tap_add_colorless};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn pump_self(p: i32, t: i32) -> Effect {
    Effect::PumpPT { what: Selector::This, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

// ── Black Widow, Natasha Romanoff (vanilla bodies) ──────────────────────────

/// Black Widow, Natasha Romanoff — {1}{R} legendary 2/2 Human Assassin Hero.
pub fn black_widow_natasha_romanoff() -> CardDefinition {
    legend(creature(
        "Black Widow, Natasha Romanoff",
        cost(&[generic(1), r()]),
        vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Hero],
        2,
        2,
    ))
}

/// Happy Hogan, Dauntless Driver — {R} legendary 2/1 Human Pilot.
pub fn happy_hogan_dauntless_driver() -> CardDefinition {
    legend(creature("Happy Hogan, Dauntless Driver", cost(&[r()]), vec![CreatureType::Human, CreatureType::Pilot], 2, 1))
}

/// Falkenrath Reaver — {1}{R} 2/2 Vampire.
pub fn falkenrath_reaver() -> CardDefinition {
    creature("Falkenrath Reaver", cost(&[generic(1), r()]), vec![CreatureType::Vampire], 2, 2)
}

/// Trained Orgg — {6}{R} 6/6 Orgg.
pub fn trained_orgg() -> CardDefinition {
    creature("Trained Orgg", cost(&[generic(6), r()]), vec![CreatureType::Orgg], 6, 6)
}

/// The Whizzer, Classic Speedster — {3}{R} legendary 3/3 Human Hero, first
/// strike, haste.
pub fn the_whizzer_classic_speedster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        ..legend(creature("The Whizzer, Classic Speedster", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3))
    }
}

// ── X-23's heroes ───────────────────────────────────────────────────────────

/// Storm, Shaker of Skies — {4}{G}{G} legendary 4/3 flying Mutant Hero. ETB:
/// destroy up to one target artifact, enchantment, or creature with flying.
pub fn storm_shaker_of_skies() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment).or(R::Creature.and(R::HasKeyword(Keyword::Flying)))),
            }),
        })],
        ..legend(creature(
            "Storm, Shaker of Skies",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            4,
            3,
        ))
    }
}

/// Wolfsbane, Highland Hero — {1}{G} legendary 2/2 trample Mutant Werewolf
/// Hero. {2}{G}: +2/+2 until end of turn, once each turn.
pub fn wolfsbane_highland_hero() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            once_per_turn: true,
            effect: pump_self(2, 2),
            ..Default::default()
        }],
        ..legend(creature(
            "Wolfsbane, Highland Hero",
            cost(&[generic(1), g()]),
            vec![CreatureType::Mutant, CreatureType::Werewolf, CreatureType::Hero],
            2,
            2,
        ))
    }
}

// ── The Howling Abomination ─────────────────────────────────────────────────

/// The Howling Abomination — {3}{R}{G} legendary 5/5 haste Human Beast
/// Warrior; trample while you've cast three or more spells this turn. When it
/// becomes the target of a spell: +2/+2 until end of turn and 2 damage to each
/// opponent.
pub fn the_howling_abomination() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Has trample as long as you've cast three or more spells this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(3) },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource).caused_by(R::IsSpellOnStack),
            effect: Effect::Seq(vec![pump_self(2, 2), deal(2, each_opponent())]),
        }],
        ..legend(creature(
            "The Howling Abomination",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Human, CreatureType::Beast, CreatureType::Warrior],
            5,
            5,
        ))
    }
}

// ── Radioactive Man ─────────────────────────────────────────────────────────

/// Radioactive Man — {4}{B} legendary 3/5 deathtouch Human Scientist Villain.
/// Combat damage to a player: that player loses half their life, rounded up.
pub fn radioactive_man() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LoseHalfLife { who: Selector::Player(PlayerRef::Target(0)), rounded_up: true },
        }],
        ..legend(creature(
            "Radioactive Man",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            3,
            5,
        ))
    }
}

/// Hexhaven Dueling Arena — Land. {T}: Add {C}. {2},{T}, sorcery speed: a
/// creature that attacked this turn becomes prepared. {4},{T}: any creature
/// becomes prepared. (Only creatures with prepare spells can.)
pub fn hexhaven_dueling_arena() -> CardDefinition {
    let prepare = |mana: u32, filter: R, sorcery_speed: bool| ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(mana)]),
        sorcery_speed,
        effect: Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::HasPrepareSpell).and(filter)),
            kind: CounterType::Prepared,
            amount: Value::ONE,
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Hexhaven Dueling Arena",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), prepare(2, R::AttackedThisTurn, true), prepare(4, R::Any, false)],
        ..Default::default()
    }
}

/// Rampart Hunter — {3}{B} 3/3 deathtouch Horror. ETB: target creature gets
/// +2/+2 and gains deathtouch until end of turn.
pub fn rampart_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Rampart Hunter", cost(&[generic(3), b()]), vec![CreatureType::Horror], 3, 3)
    }
}

/// Extended Absence — {3}{B} instant. Exile target creature or planeswalker;
/// 1 damage to each opponent and you gain 1 life.
pub fn extended_absence() -> CardDefinition {
    CardDefinition {
        name: "Extended Absence",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature.or(R::Planeswalker)) },
            deal(1, each_opponent()),
            gain_life(1),
        ]),
        ..Default::default()
    }
}

// ── Rhino, Terrible Trampler / Abomination, Irradiated Brute ────────────────

/// Rhino, Terrible Trampler — {4}{G}{G} legendary 7/6 trample Human Villain.
/// ETB: destroy target artifact or land; distribute three +1/+1 counters among
/// up to three other target creatures, which gain trample until end of turn.
/// (Two triggers — the destroy's target and the distribution's slots would
/// collide in one `Seq`.)
pub fn rhino_terrible_trampler() -> CardDefinition {
    let trample = |n| Effect::GrantKeyword { what: Selector::Target(n), keyword: Keyword::Trample, duration: Duration::EndOfTurn };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::Destroy { what: target_filtered(R::Artifact.or(R::Land)) }),
            etb(Effect::Seq(vec![
                Effect::DistributeCounters {
                    total: Value::Const(3),
                    counter: CounterType::PlusOnePlusOne,
                    filter: R::Creature.and(R::OtherThanSource),
                    max_targets: 3,
                },
                trample(0),
                trample(1),
                trample(2),
            ])),
        ],
        ..legend(creature(
            "Rhino, Terrible Trampler",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Human, CreatureType::Villain],
            7,
            6,
        ))
    }
}

/// Dominion Supervisor — {2}{G} 3/1 Vorta Doctor. ETB: a +1/+1 counter on
/// target creature. Your creatures with counters on them have trample.
pub fn dominion_supervisor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with counters on them have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::WithAnyCounter)),
                keyword: Keyword::Trample,
            },
        }],
        ..creature("Dominion Supervisor", cost(&[generic(2), g()]), vec![CreatureType::Vorta, CreatureType::Doctor], 3, 1)
    }
}

/// Aspect of Wolf — {1}{G} Aura. Enchanted creature gets +X/+Y: X is half
/// your Forests rounded down, Y half rounded up.
pub fn aspect_of_wolf() -> CardDefinition {
    let forests = || Box::new(Value::count(Selector::EachPermanent(R::HasLandType(LandType::Forest).and(R::ControlledByYou))));
    CardDefinition {
        name: "Aspect of Wolf",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +X/+Y, X half your Forests rounded down, Y rounded up.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::HalvedRoundDown(forests()),
                toughness: Value::HalvedRoundUp(forests()),
            },
        }],
        ..Default::default()
    }
}

/// Living Lands — {3}{G} enchantment. All Forests are 1/1 creatures that are
/// still lands.
pub fn living_lands() -> CardDefinition {
    CardDefinition {
        name: "Living Lands",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "All Forests are 1/1 creatures that are still lands.",
            effect: StaticEffect::MatchingLandsAreCreatures {
                filter: R::HasLandType(LandType::Forest),
                power: 1,
                toughness: 1,
                keywords: vec![],
                creature_types: vec![],
                colors: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Powerful Broker — {2}{G} 3/3 Human Villain. {T}, sorcery speed: for each
/// kind of counter on target permanent or player, another of that kind.
pub fn powerful_broker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounterOfEachKindOn { what: target_filtered(R::Permanent.or(R::Player)) },
            ..Default::default()
        }],
        ..creature("Powerful Broker", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Villain], 3, 3)
    }
}

/// Serpent Specialist — {G} 1/1 deathtouch Human Snake Villain. Power-up —
/// {3}{G}: two +1/+1 counters on it.
pub fn serpent_specialist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            power_up: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature("Serpent Specialist", cost(&[g()]), vec![CreatureType::Human, CreatureType::Snake, CreatureType::Villain], 1, 1)
    }
}

/// Undercover Skrull — {1}{G} 1/1 Skrull Shapeshifter Villain. With two or
/// more creature cards in your graveyard it gets +2/+2 and is every creature
/// type (a granted changeling, CR 702.73a). {T}: one mana of any color.
pub fn undercover_skrull() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as there are two or more creature cards in your graveyard, this gets +2/+2 and is all creature types.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
                    Value::Const(2),
                ),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Changeling],
            },
        }],
        activated_abilities: vec![tap_add_any_color()],
        ..creature(
            "Undercover Skrull",
            cost(&[generic(1), g()]),
            vec![CreatureType::Skrull, CreatureType::Shapeshifter, CreatureType::Villain],
            1,
            1,
        )
    }
}

// ── Venom, Deadly Devourer / Symbiote Spider-Man ────────────────────────────

/// Grendel, Spawn of Knull — {3}{B} legendary 3/3 flying deathtouch Symbiote
/// Dragon Villain.
pub fn grendel_spawn_of_knull() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        ..legend(creature(
            "Grendel, Spawn of Knull",
            cost(&[generic(3), b()]),
            vec![CreatureType::Symbiote, CreatureType::Dragon, CreatureType::Villain],
            3,
            3,
        ))
    }
}

/// Venom, Evil Unleashed — {4}{B} legendary 4/5 deathtouch Symbiote Villain.
/// {2}{B}, exile it from your graveyard, sorcery speed: two +1/+1 counters on
/// target creature, which gains deathtouch until end of turn.
pub fn venom_evil_unleashed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..legend(creature("Venom, Evil Unleashed", cost(&[generic(4), b()]), vec![CreatureType::Symbiote, CreatureType::Villain], 4, 5))
    }
}

/// Alien Symbiosis — {1}{B} Aura. Enchanted creature gets +1/+1, has menace,
/// and is a Symbiote. Castable from your graveyard by also discarding a card.
pub fn alien_symbiosis() -> CardDefinition {
    CardDefinition {
        name: "Alien Symbiosis",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::GraveyardCast],
        flashback_additional_cost: vec![AdditionalCastCost::Discard { count: 1, filter: None }],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Menace],
            add_creature_types: vec![CreatureType::Symbiote],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Venom's Hunger — {4}{B} sorcery, {2} less with a Villain: destroy target
/// creature, gain 2 life.
pub fn venoms_hunger() -> CardDefinition {
    CardDefinition {
        name: "Venom's Hunger",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if you control a Villain.",
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 2,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Villain).and(R::ControlledByYou),
                )),
            },
        }],
        effect: Effect::Seq(vec![Effect::Destroy { what: target_filtered(R::Creature) }, gain_life(2)]),
        ..Default::default()
    }
}

/// Spider-Man No More — {1}{U} Aura. Enchanted creature is a Citizen with
/// base 1/1, has defender and loses all other abilities and creature types.
pub fn spider_man_no_more() -> CardDefinition {
    CardDefinition {
        name: "Spider-Man No More",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((1, 1)),
            set_creature_types: Some(vec![CreatureType::Citizen]),
            remove_abilities: true,
            keywords: vec![Keyword::Defender],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Risky Research — {2}{B} sorcery: surveil 2, draw two, lose 2 life.
pub fn risky_research() -> CardDefinition {
    CardDefinition {
        name: "Risky Research",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

// ── Speed, Young Avenger ────────────────────────────────────────────────────

/// Awaken the Ancient — {1}{R}{R}{R} Aura, enchant Mountain. Enchanted
/// Mountain is a 7/7 red Giant creature with haste. It's still a land.
pub fn awaken_the_ancient() -> CardDefinition {
    CardDefinition {
        name: "Awaken the Ancient",
        cost: cost(&[generic(1), r(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::HasLandType(LandType::Mountain)) },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((7, 7)),
            add_card_types: vec![CardType::Creature],
            add_creature_types: vec![CreatureType::Giant],
            set_colors: Some(vec![Color::Red]),
            keywords: vec![Keyword::Haste],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Super Speed — {R} Aura with flash. ETB: enchanted creature gains first
/// strike until end of turn. Enchanted creature gets +1/+0 and has haste.
pub fn super_speed() -> CardDefinition {
    CardDefinition {
        name: "Super Speed",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        })],
        equipped_bonus: Some(EquipBonus { power: 1, keywords: vec![Keyword::Haste], ..Default::default() }),
        ..Default::default()
    }
}

/// Smashing Spree — {1}{R} instant: target attacking creature gets +3/+3 and
/// gains trample until end of turn.
pub fn smashing_spree() -> CardDefinition {
    CardDefinition {
        name: "Smashing Spree",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

// ── Nadaar, Selfless Paladin ────────────────────────────────────────────────

/// Goliath Paladin — {4}{W} 3/6 vigilance Giant Knight. ETB: you take the
/// initiative.
pub fn goliath_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::TakeInitiative { who: PlayerRef::You })],
        ..creature("Goliath Paladin", cost(&[generic(4), w()]), vec![CreatureType::Giant, CreatureType::Knight], 3, 6)
    }
}

/// Planar Ally — {3}{W}{W} 3/3 flying Angel. Attacks: venture into the
/// dungeon.
pub fn planar_ally() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Venture)],
        ..creature("Planar Ally", cost(&[generic(3), w(), w()]), vec![CreatureType::Angel], 3, 3)
    }
}

/// Veteran Dungeoneer — {3}{W} 3/4 Human Warrior. ETB: venture into the
/// dungeon.
pub fn veteran_dungeoneer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Venture)],
        ..creature("Veteran Dungeoneer", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 4)
    }
}

/// Undercellar Sweep — {4}{W} enchantment. ETB: you take the initiative.
/// Whenever you attack, if you or a player you're attacking has the
/// initiative, two 1/1 white Soldiers tapped and attacking (CR 603.4).
pub fn undercellar_sweep() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let initiative = Predicate::Any(vec![
        Predicate::HasInitiative { who: PlayerRef::You },
        Predicate::HasInitiative { who: PlayerRef::EachPlayerYouAreAttacking },
    ]);
    CardDefinition {
        name: "Undercellar Sweep",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::TakeInitiative { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(initiative),
                effect: Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: Arc::new(soldier),
                    cleanup: Default::default(),
                    defender: Some(PlayerRef::EachPlayerYouAreAttacking),
                },
            },
        ],
        ..Default::default()
    }
}

// ── Wraith, Vicious Vigilante / Spider-Woman, Stunning Savior ───────────────

/// Wraith, Vicious Vigilante — {1}{W}{U} legendary 1/1 Human Detective Hero,
/// double strike; Fear Gas — can't be blocked.
pub fn wraith_vicious_vigilante() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Unblockable],
        ..legend(creature(
            "Wraith, Vicious Vigilante",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Detective, CreatureType::Hero],
            1,
            1,
        ))
    }
}

/// SP//dr, Piloted by Peni — {3}{W}{U} legendary 4/4 vigilance artifact
/// Spider Hero. ETB: a +1/+1 counter on target creature. A modified creature
/// of yours dealing combat damage to a player draws you a card.
pub fn sp_dr_piloted_by_peni() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::IsModified) },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..legend(creature(
            "SP//dr, Piloted by Peni",
            cost(&[generic(3), w(), u()]),
            vec![CreatureType::Spider, CreatureType::Hero],
            4,
            4,
        ))
    }
}

/// Spider-Woman, Stunning Savior — {1}{W/U} legendary 2/2 flying Spider Human
/// Hero. Venom Blast — opponents' artifacts and creatures enter tapped.
pub fn spider_woman_stunning_savior() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Artifacts and creatures your opponents control enter tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::EachPermanent(R::ControlledByOpponent.and(R::Artifact.or(R::Creature))),
            },
        }],
        ..legend(creature(
            "Spider-Woman, Stunning Savior",
            cost(&[generic(1), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            2,
            2,
        ))
    }
}

/// Sun-Spider, Nimble Webber — {3}{W/U} legendary 3/2 Spider Human Hero with
/// flying during your turn. ETB: tutor an Aura or Equipment card to hand.
pub fn sun_spider_nimble_webber() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, Sun-Spider has flying.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Flying }),
            },
        }],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..legend(creature(
            "Sun-Spider, Nimble Webber",
            cost(&[generic(3), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            3,
            2,
        ))
    }
}

/// University Campus — land, enters tapped. {T}: {W} or {U}. {4},{T}: surveil 1.
pub fn university_campus() -> CardDefinition {
    let mut d = crate::sets::dual_land_untyped("University Campus", Color::White, Color::Blue, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d.activated_abilities.push(ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(4)]),
        effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ..Default::default()
    });
    d
}

// ── Marvel's white Heroes (Matt Murdock, The Wasp, Patriot, War Machine) ─────

fn hero() -> R {
    R::HasCreatureType(CreatureType::Hero)
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn eot_keyword(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn your_combat(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl), effect }
}

/// Matt Murdock, Justice Seeker — {1}{W} legendary 2/2 Human Advisor Hero.
/// Beginning of combat on your turn: you may pay {1}; when you do, a +1/+1
/// counter on target creature you control. Your creatures with counters have
/// ward {1}.
pub fn matt_murdock_justice_seeker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_combat(Effect::MayPay {
            description: "Pay {1} to put a +1/+1 counter on a creature you control?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::Reflexive {
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(yours(R::Creature)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            }),
            else_: None,
        })],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a counter on it has ward {1}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Creature).and(R::WithAnyCounter)),
                keyword: Keyword::Ward(WardCost::Mana(cost(&[generic(1)]))),
            },
        }],
        ..legend(creature(
            "Matt Murdock, Justice Seeker",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Advisor, CreatureType::Hero],
            2,
            2,
        ))
    }
}

/// Foggy Nelson, On Retainer — {2}{W} legendary 1/3 flash Human Advisor. ETB:
/// a +1/+1 counter on another target creature you control; it gains hexproof
/// until end of turn.
pub fn foggy_nelson_on_retainer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            eot_keyword(Selector::Target(0), Keyword::Hexproof),
        ]))],
        ..legend(creature("Foggy Nelson, On Retainer", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 3))
    }
}

/// Murdock's Crusade — {1}{W} sorcery, teamwork 4. Choose one (both with
/// teamwork): exile target creature with toughness 4+; exile target
/// enchantment with mana value 4+.
pub fn murdocks_crusade() -> CardDefinition {
    CardDefinition {
        name: "Murdock's Crusade",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower { filter: yours(R::Creature), power: 4 }),
        modes_widen: Some(ModesWiden::Kicked),
        effect: Effect::ChooseModesCast {
            min: 1,
            max: 1,
            allow_repeats: false,
            modes: vec![
                Effect::Exile { what: target_filtered(R::Creature.and(R::ToughnessAtLeast(4))) },
                Effect::Exile { what: target_filtered(R::Enchantment.and(R::ManaValueAtLeast(4))) },
            ],
        },
        ..Default::default()
    }
}

/// The Wasp, Janet Van Dyne — {2}{W} legendary 1/1 flying Human Hero. ETB: 4
/// damage to target tapped creature an opponent controls.
pub fn the_wasp_janet_van_dyne() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)),
            amount: Value::Const(4),
        })],
        ..legend(creature("The Wasp, Janet Van Dyne", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Hero], 1, 1))
    }
}

/// Wondrous Revival — {4}{W}{W} sorcery: return up to three target Hero
/// creature cards from your graveyard to the battlefield.
pub fn wondrous_revival() -> CardDefinition {
    CardDefinition {
        name: "Wondrous Revival",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature.and(hero()).and(R::InYourGraveyard),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
        ..Default::default()
    }
}

/// Clockwork Drawbridge — {W} 0/3 defender artifact creature Wall. {2}{W},
/// {T}: tap target creature.
pub fn clockwork_drawbridge() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Clockwork Drawbridge", cost(&[w()]), vec![CreatureType::Wall], 0, 3)
    }
}

/// Origin of the Avengers — {1}{W} Saga. I: scry 2. II: you may put a Hero
/// creature card with mana value 3 or less from your hand onto the
/// battlefield; if you don't, draw a card. III: a +1/+1 counter on each
/// creature you control.
pub fn origin_of_the_avengers() -> CardDefinition {
    CardDefinition {
        name: "Origin of the Avengers",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            (
                2,
                Effect::MayDoElse {
                    description: "Put a Hero with mana value 3 or less from your hand onto the battlefield? If you don't, draw a card.".into(),
                    body: Box::new(Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::Creature.and(hero()).and(R::ManaValueAtMost(3)),
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    }),
                    else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ),
            (
                3,
                Effect::AddCounter {
                    what: Selector::EachPermanent(yours(R::Creature)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Bucky Barnes, Eager Ally — {1}{W} legendary 2/2 vigilance Human Soldier
/// Hero. Dies: look at the top four, you may take an Equipment, Hero or
/// Soldier card; the rest go to the bottom in a random order.
pub fn bucky_barnes_eager_ally() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_dies(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(
                R::HasArtifactSubtype(ArtifactSubtype::Equipment)
                    .or(hero())
                    .or(R::HasCreatureType(CreatureType::Soldier)),
            ),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))],
        ..legend(creature(
            "Bucky Barnes, Eager Ally",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            2,
            2,
        ))
    }
}

/// Captain America, Steve Rogers — {3}{W}{W} legendary 4/4 lifelink Human
/// Soldier Hero. Attacks: a +1/+1 counter on another target creature you
/// control; it gains indestructible until end of turn.
pub fn captain_america_steve_rogers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            eot_keyword(Selector::Target(0), Keyword::Indestructible),
        ]))],
        ..legend(creature(
            "Captain America, Steve Rogers",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            4,
            4,
        ))
    }
}

/// Angelic Guardian — {4}{W}{W} 5/5 flying Angel. Whenever one or more
/// creatures you control attack, they gain indestructible until end of turn.
pub fn angelic_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_you_attack(eot_keyword(
            Selector::EachPermanent(yours(R::Creature).and(R::IsAttacking)),
            Keyword::Indestructible,
        ))],
        ..creature("Angelic Guardian", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 5, 5)
    }
}

/// War Machine, Legacy of Iron — {2}{R/W} legendary 1/3 flying artifact
/// creature Human Hero. Beginning of combat on your turn: another target
/// creature you control gets +X/+0, X its power.
pub fn war_machine_legacy_of_iron() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![your_combat(Effect::PumpPT {
            what: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
            power: Value::PowerOf(Box::new(Selector::This)),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..legend(creature(
            "War Machine, Legacy of Iron",
            cost(&[generic(2), hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Human, CreatureType::Hero],
            1,
            3,
        ))
    }
}

/// Captain America, Wings of Freedom — {2}{W} legendary 3/1 Human Soldier
/// Hero; flying, first strike, ward {1}. Attacks: each other Hero you control
/// gets +X/+X, X its toughness.
pub fn captain_america_wings_of_freedom() -> CardDefinition {
    let x = || Value::ToughnessOf(Box::new(Selector::This));
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(yours(R::Creature.and(hero())).and(R::OtherThanSource)),
            power: x(),
            toughness: x(),
            duration: Duration::EndOfTurn,
        })],
        ..legend(creature(
            "Captain America, Wings of Freedom",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            1,
        ))
    }
}

/// Doctor Spectrum — {3}{W}{W} legendary 3/3 flying Human Hero. ETB, choose
/// one: a 0/4 defender Wall token; a +1/+1 counter on each other Hero you
/// control; destroy target enchantment.
pub fn doctor_spectrum() -> CardDefinition {
    let wall = TokenDefinition {
        name: "Wall".into(),
        power: 0,
        toughness: 4,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        keywords: vec![Keyword::Defender],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(wall) },
            Effect::AddCounter {
                what: Selector::EachPermanent(yours(R::Creature.and(hero())).and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Destroy { what: target_filtered(R::Enchantment) },
        ]))],
        ..legend(creature("Doctor Spectrum", cost(&[generic(3), w(), w()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3))
    }
}

/// Hero in Training — {2}{W} 2/2 Human Hero. ETB: draw a card; with another
/// Hero, gain 2 life.
pub fn hero_in_training() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(yours(R::Creature.and(hero())).and(R::OtherThanSource))),
                then: Box::new(gain_life(2)),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Hero in Training", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Hero], 2, 2)
    }
}

/// Borough Backup — {4}{W} sorcery: two 3/2 white vigilance Hero tokens.
/// Basic landcycling {2}.
pub fn borough_backup() -> CardDefinition {
    let hero_token = TokenDefinition {
        name: "Hero".into(),
        power: 3,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Borough Backup",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))],
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(hero_token) },
        ..Default::default()
    }
}

/// Captain America, Liberator — {3}{W}{W} legendary 5/5 Human Soldier Hero.
/// ETB: you may search for an Equipment card with mana value 3 or less and put
/// it onto the battlefield. Attacks: a 1/1 white Soldier per Equipment
/// attached to him.
pub fn captain_america_liberator() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let equipment = Selector::MatchingAmong {
        inner: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
        filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ManaValueAtMost(3)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            on_attack(Effect::CreateToken { who: PlayerRef::You, count: Value::count(equipment), definition: Arc::new(soldier) }),
        ],
        ..legend(creature(
            "Captain America, Liberator",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            5,
            5,
        ))
    }
}

/// Iconic Shield — {W} Equipment, equip {3}. Equipped creature gets +1/+2 and
/// has "whenever this attacks, another target attacking creature gains
/// indestructible until end of turn".
pub fn iconic_shield() -> CardDefinition {
    CardDefinition {
        name: "Iconic Shield",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 2,
            triggered_abilities: vec![on_attack(eot_keyword(
                target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                Keyword::Indestructible,
            ))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// The Falcon, Sam Wilson — {4}{W} legendary 3/3 flying Human Hero. ETB: a
/// +1/+1 counter on each other creature you control.
pub fn the_falcon_sam_wilson() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(yours(R::Creature).and(R::OtherThanSource)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..legend(creature("The Falcon, Sam Wilson", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3))
    }
}

// ── Lady Sun ────────────────────────────────────────────────────────────────

/// "Activate only during your turn, before attackers are declared."
fn your_turn_before_attackers() -> Predicate {
    Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::Any(vec![
            Predicate::CurrentStepIs(TurnStep::Untap),
            Predicate::CurrentStepIs(TurnStep::Upkeep),
            Predicate::CurrentStepIs(TurnStep::Draw),
            Predicate::CurrentStepIs(TurnStep::PreCombatMain),
            Predicate::CurrentStepIs(TurnStep::BeginCombat),
        ]),
    ])
}

/// Lady Sun — {1}{U}{U} legendary 1/1 Human Advisor. {T}: return Lady Sun and
/// another target creature to their owners' hands; only during your turn,
/// before attackers are declared.
pub fn lady_sun() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(your_turn_before_attackers()),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
            ]),
            ..Default::default()
        }],
        ..legend(creature("Lady Sun", cost(&[generic(1), u(), u()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 1))
    }
}

/// Imperial Cosmographer — {2}{U} 2/3 vigilance Kree Scientist. Another
/// creature of yours leaving the battlefield without dying: two +1/+1
/// counters on it.
pub fn imperial_cosmographer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureLeavesBattlefieldNotDying, EventScope::AnotherOfYours),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
        }],
        ..creature("Imperial Cosmographer", cost(&[generic(2), u()]), vec![CreatureType::Kree, CreatureType::Scientist], 2, 3)
    }
}

/// Puca's Mischief — {3}{U} enchantment. Your upkeep: you may exchange
/// control of target nonland permanent you control and target nonland
/// permanent an opponent controls with equal or lesser mana value.
pub fn pucas_mischief() -> CardDefinition {
    CardDefinition {
        name: "Puca's Mischief",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exchange control of your nonland permanent and an opponent's cheaper one?".into(),
                body: Box::new(Effect::ExchangeControl {
                    a: Selector::TargetFiltered { slot: 0, filter: R::Permanent.and(R::Not(Box::new(R::Land))).and(R::ControlledByYou) },
                    b: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Permanent
                            .and(R::Not(Box::new(R::Land)))
                            .and(R::ControlledByOpponent)
                            .and(R::ManaValueAtMostTargetSlot(0)),
                    },
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Seymour Flux ────────────────────────────────────────────────────────────

/// Gurgling Anointer — {2}{B} 1/3 flying Phyrexian Horror. Your second draw
/// each turn: a +1/+1 counter on it. Dies: return another target creature
/// card with mana value ≤ its power from your graveyard to the battlefield.
pub fn gurgling_anointer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            on_dies(Effect::Move {
                what: target_filtered(
                    R::Creature.and(R::InYourGraveyard).and(R::OtherThanSource).and(R::ManaValueAtMostSourcePower),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        ],
        ..creature("Gurgling Anointer", cost(&[generic(2), b()]), vec![CreatureType::Phyrexian, CreatureType::Horror], 1, 3)
    }
}

/// Heirloom Auntie — {2}{B} 4/4 Goblin Warlock entering with two -1/-1
/// counters. Another creature of yours dying: surveil 1, then remove a -1/-1
/// counter from it.
pub fn heirloom_auntie() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Seq(vec![
                Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                Effect::RemoveCounter { what: Selector::This, kind: CounterType::MinusOneMinusOne, amount: Value::ONE },
            ]),
        }],
        ..creature("Heirloom Auntie", cost(&[generic(2), b()]), vec![CreatureType::Goblin, CreatureType::Warlock], 4, 4)
    }
}

/// Bogslither's Embrace — {1}{B} sorcery; blight 1 or pay {3} as an
/// additional cost. Exile target creature.
pub fn bogslithers_embrace() -> CardDefinition {
    CardDefinition {
        name: "Bogslither's Embrace",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::BlightOrPay { n: 1, pay: 3 }],
        effect: Effect::Exile { what: target_filtered(R::Creature) },
        ..Default::default()
    }
}

// ── Arcum Dagsson ───────────────────────────────────────────────────────────

/// Arcum Dagsson — {3}{U} legendary 2/2 Human Artificer. {T}: target artifact
/// creature's controller sacrifices it, then may search for a noncreature
/// artifact card and put it onto the battlefield.
pub fn arcum_dagsson() -> CardDefinition {
    let them = || PlayerRef::ControllerOf(Box::new(Selector::Target(0)));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::SacrificePermanent { what: target_filtered(R::Artifact.and(R::Creature)) },
                Effect::Search {
                    who: them(),
                    filter: R::Artifact.and(R::Not(Box::new(R::Creature))),
                    to: ZoneDest::Battlefield { controller: them(), tapped: false },
                },
            ]),
            ..Default::default()
        }],
        ..legend(creature("Arcum Dagsson", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 2))
    }
}

/// Combat Courier — {1} 1/1 artifact creature Construct. {2}, sacrifice it:
/// draw a card. Unearth {U}.
pub fn combat_courier() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            unearth(cost(&[u()])),
        ],
        ..creature("Combat Courier", cost(&[generic(1)]), vec![CreatureType::Construct], 1, 1)
    }
}

/// Filigree Sages — {3}{U} 2/3 artifact creature Vedalken Wizard. {2}{U}:
/// untap target artifact.
pub fn filigree_sages() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Untap { what: target_filtered(R::Artifact), up_to: None },
            ..Default::default()
        }],
        ..creature("Filigree Sages", cost(&[generic(3), u()]), vec![CreatureType::Vedalken, CreatureType::Wizard], 2, 3)
    }
}

// ── Black Widow, Intel Expert / Justice, Vance Astrovik / Atogatog ──────────

/// Black Panther, Claws of Bast — {2}{W} legendary 3/3 lifelink Human Warrior
/// Hero. A creature of yours attacking alone gets a +1/+1 counter.
pub fn black_panther_claws_of_bast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::AttackingAlone),
            effect: Effect::AddCounter { what: Selector::TriggerSource, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend(creature(
            "Black Panther, Claws of Bast",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            3,
            3,
        ))
    }
}

/// Cache Raiders — {3}{U}{U} 4/4 Merfolk Rogue. Your upkeep: return a
/// permanent you control to its owner's hand (chosen, not targeted).
pub fn cache_raiders() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ReturnOneYouControl { filter: R::Permanent, keep_best: false },
        }],
        ..creature("Cache Raiders", cost(&[generic(3), u(), u()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 4, 4)
    }
}

/// Disruptor of Currents — {3}{U}{U} 3/3 flash, convoke Merfolk Wizard. ETB:
/// return up to one other target nonland permanent to its owner's hand.
pub fn disruptor_of_currents() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Permanent.and(R::Not(Box::new(R::Land))).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        ..creature("Disruptor of Currents", cost(&[generic(3), u(), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 3, 3)
    }
}

/// Voidmage Husher — {3}{U} 2/2 flash Human Wizard. ETB: counter target
/// activated ability. Whenever you cast a spell, you may return it to its
/// owner's hand.
pub fn voidmage_husher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::CounterAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::Not(Box::new(R::HasTriggeredAbilityOnStack)))),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Return Voidmage Husher to its owner's hand?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                    }),
                },
            },
        ],
        ..creature("Voidmage Husher", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Chronatog Totem — {3} artifact. {T}: add {U}. {1}{U}: a 1/2 blue Atog
/// artifact creature until end of turn. {0}, once each turn while a creature:
/// +3/+3 until end of turn and you skip your next turn.
pub fn chronatog_totem() -> CardDefinition {
    let animate = Effect::BecomeCreature {
        what: Selector::This,
        power: Value::Const(1),
        toughness: Value::Const(2),
        creature_types: vec![CreatureType::Atog],
        keywords: vec![],
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Chronatog Totem",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                effect: crate::effect::shortcut::colored_animation(animate, &[Color::Blue]),
                ..Default::default()
            },
            ActivatedAbility {
                once_per_turn: true,
                condition: Some(Predicate::EntityMatches { what: Selector::This, filter: R::Creature }),
                effect: Effect::Seq(vec![
                    Effect::PumpPT { what: Selector::This, power: Value::Const(3), toughness: Value::Const(3), duration: Duration::EndOfTurn },
                    Effect::SkipTurns { who: PlayerRef::You, count: Value::ONE },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Barren Glory — {4}{W}{W} enchantment. Your upkeep, if you control no other
/// permanents and have no cards in hand: you win the game (CR 603.4).
pub fn barren_glory() -> CardDefinition {
    let bare = Predicate::All(vec![
        Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
            R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource),
        )))),
        Predicate::ValueEquals(Value::HandSizeOf(PlayerRef::You), Value::Const(0)),
    ]);
    CardDefinition {
        name: "Barren Glory",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(bare.clone()),
            effect: Effect::If { cond: bare, then: Box::new(Effect::WinGame { who: PlayerRef::You }), else_: Box::new(Effect::Noop) },
        }],
        ..Default::default()
    }
}
