//! Commander: the cards the **Turtle Power!** precon (TMC, Heroes in a Half
//! Shell) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_heroes.rs`.
//!
//! Residuals (each also on its card):
//! - **Heroes in a Half Shell** — "each of those creatures" is each Mutant,
//!   Ninja or Turtle of yours that dealt damage to a player this turn.
//! - **Coin of Mastery** — artifact mana is counted from the pool, so mana
//!   floated from lands and artifacts together can read low.
//! - **Double Jump // Flying Kick** — cast fused, Flying Kick has no enemy
//!   target (a fused right half reads one target).
//! - **Special Move** — Foot Toss's creature is your greatest-power creature,
//!   chosen on resolution rather than targeted.
//! - **Vigor** — the prevention is a replacement, so "damage can't be
//!   prevented" doesn't stop it.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, SplitCard, SplitHalf,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    etb, evolve, investigate, on_attack, on_dies, partner_with_search, squad_etb, target_any, target_filtered,
};
use crate::effect::{AttackingTokenCleanup, Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w};
use crate::sets::{tap_add_colorless, tap_add_commander_identity};
use crabomination_base::tokens::{food_token, mutagen_token, treasure_token};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn land(name: &'static str) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Land], ..Default::default() }
}

fn enters_tapped() -> StaticAbility {
    StaticAbility {
        description: "This land enters tapped.",
        effect: StaticEffect::EntersTapped { applies_to: Selector::This },
    }
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn plus(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn keyword_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn create(definition: TokenDefinition, count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(definition) }
}

fn character_select() -> Keyword {
    Keyword::PartnerLabel("Character select".into())
}

/// A Mutant, Ninja or Turtle.
fn tmnt() -> R {
    R::HasCreatureType(CreatureType::Mutant)
        .or(R::HasCreatureType(CreatureType::Ninja))
        .or(R::HasCreatureType(CreatureType::Turtle))
}

/// "Whenever [a permanent of yours matching `filter`] enters, [effect]."
fn on_yours_enters(scope: EventScope, filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, scope)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

/// "Whenever [this or another] nontoken creature you control leaves the
/// battlefield, [effect]."
fn on_nontoken_leaves(filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

fn on_combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

/// "At the beginning of your second main phase, [effect]."
fn your_second_main(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl),
        effect,
    }
}

/// "Target creature you control deals damage equal to its power to target
/// creature an opponent controls" (Flying Kick, Super Combo).
fn bite() -> Effect {
    Effect::DealDamageEqualToPower {
        source: target_filtered(yours(R::Creature)),
        target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
    }
}

fn black_token(name: &str, kind: CreatureType) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![kind], ..Default::default() },
        ..Default::default()
    }
}

fn rat_token() -> TokenDefinition {
    black_token("Rat", CreatureType::Rat)
}

fn ninja_token() -> TokenDefinition {
    black_token("Ninja", CreatureType::Ninja)
}

fn robot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Robot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    }
}

// ── Commander ───────────────────────────────────────────────────────────────

/// Heroes in a Half Shell — a five-color 5/5 whose Mutants, Ninjas and
/// Turtles grow and draw when they connect.
///
/// ⚠ Residual: "each of those creatures" is each such creature of yours that
/// dealt damage to a player this turn (no selector names one batch's
/// damage sources).
pub fn heroes_in_a_half_shell() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Menace, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .once_per_batch()
                .dealt_by(yours(tmnt())),
            effect: Effect::Seq(vec![
                plus(
                    Selector::EachPermanent(yours(R::Creature.and(tmnt())).and(R::DamagedAPlayerThisTurn)),
                    Value::ONE,
                ),
                draw(Value::ONE),
            ]),
        }],
        ..creature(
            "Heroes in a Half Shell",
            cost(&[w(), u(), b(), r(), g()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            5,
            5,
        )
    })
}

// ── The partners ────────────────────────────────────────────────────────────

/// April O'Neil, Live on the Scene — investigate as a Mutant, Ninja or
/// Turtle of yours enters.
pub fn april_oneil_live_on_the_scene() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![character_select()],
        triggered_abilities: vec![on_yours_enters(EventScope::YourControl, tmnt(), investigate(1))],
        ..creature(
            "April O'Neil, Live on the Scene",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            1,
        )
    })
}

/// Donatello, the Brains — every token creation of yours adds a Mutagen.
pub fn donatello_the_brains() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![character_select()],
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, those tokens plus a Mutagen token are created instead.",
            effect: StaticEffect::TokenCreationAddsToken { definition: mutagen_token() },
        }],
        ..creature(
            "Donatello, the Brains",
            cost(&[generic(2), u()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            2,
            4,
        )
    })
}

/// Leonardo, the Balance — a token of yours entering may counter up your
/// team once a turn; WUBRG gives the team menace, trample and lifelink.
pub fn leonardo_the_balance() -> CardDefinition {
    let team = || Selector::EachPermanent(yours(R::Creature));
    legendary(CardDefinition {
        keywords: vec![character_select()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsToken })
                .once_per_turn(),
            effect: may("Put a +1/+1 counter on each creature you control?", plus(team(), Value::ONE)),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            effect: Effect::Seq(vec![
                keyword_eot(team(), Keyword::Menace),
                keyword_eot(team(), Keyword::Trample),
                keyword_eot(team(), Keyword::Lifelink),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Leonardo, the Balance",
            cost(&[generic(3), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            3,
            3,
        )
    })
}

/// Michelangelo, the Heart — after an attack, a counter and a Food.
pub fn michelangelo_the_heart() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Trample, character_select()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl)
                .with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            effect: Effect::Seq(vec![plus(target_filtered(R::Creature), Value::ONE), create(food_token(), Value::ONE)]),
        }],
        ..creature(
            "Michelangelo, the Heart",
            cost(&[generic(1), g()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            2,
            1,
        )
    })
}

/// Raphael, the Muscle — creatures of yours with counters deal double
/// damage; a Mutagen as it enters.
pub fn raphael_the_muscle() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![character_select()],
        static_abilities: vec![StaticAbility {
            description: "Double all damage that creatures you control with counters on them would deal.",
            effect: StaticEffect::DoubleDamageFromControlledMatching { filter: R::Creature.and(R::WithAnyCounter) },
        }],
        triggered_abilities: vec![etb(create(mutagen_token(), Value::ONE))],
        ..creature(
            "Raphael, the Muscle",
            cost(&[generic(4), r()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            4,
            4,
        )
    })
}

/// Splinter, the Mentor — a Mutagen whenever it or another nontoken creature
/// of yours leaves.
pub fn splinter_the_mentor() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Menace, character_select()],
        triggered_abilities: vec![on_nontoken_leaves(R::Creature.and(R::NotToken), create(mutagen_token(), Value::ONE))],
        ..creature(
            "Splinter, the Mentor",
            cost(&[generic(1), b()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Rat],
            2,
            2,
        )
    })
}

/// Lita, Little Orphan Amphibian — alliance, a mode not yet chosen this turn.
pub fn lita_little_orphan_amphibian() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![on_yours_enters(
            EventScope::AnotherOfYours,
            R::Creature,
            Effect::ChooseUnchosenModeThisTurn {
                modes: vec![
                    plus(Selector::This, Value::ONE),
                    create(food_token(), Value::ONE),
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                ],
            },
        )],
        ..creature(
            "Lita, Little Orphan Amphibian",
            cost(&[generic(1), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            2,
            1,
        )
    })
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Baxter, Fly in the Ointment — your countered creatures fly as it enters
/// or attacks; grows as you draw.
pub fn baxter_fly_in_the_ointment() -> CardDefinition {
    let fly = || keyword_eot(Selector::EachPermanent(yours(R::Creature).and(R::WithAnyCounter)), Keyword::Flying);
    legendary(CardDefinition {
        triggered_abilities: vec![
            etb(fly()),
            on_attack(fly()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: plus(Selector::This, Value::ONE),
            },
        ],
        ..creature(
            "Baxter, Fly in the Ointment",
            cost(&[generic(3), u()]),
            vec![CreatureType::Insect, CreatureType::Mutant, CreatureType::Scientist],
            2,
            2,
        )
    })
}

/// Bebop, Skull & Crossbones — deathtouch; may draw X and lose X on a hit,
/// X its counters.
pub fn bebop_skull_crossbones() -> CardDefinition {
    let x = || Value::TotalCountersOn { what: Box::new(Selector::This) };
    legendary(CardDefinition {
        keywords: vec![Keyword::PartnerWith("Rocksteady, Mutant Marauder".into()), Keyword::Deathtouch],
        triggered_abilities: vec![
            partner_with_search("Rocksteady, Mutant Marauder"),
            on_combat_damage_to_player(may(
                "Draw a card for each counter on Bebop and lose that much life?",
                Effect::Seq(vec![draw(x()), Effect::LoseLife { who: Selector::You, amount: x() }]),
            )),
        ],
        ..creature(
            "Bebop, Skull & Crossbones",
            cost(&[generic(1), b()]),
            vec![CreatureType::Boar, CreatureType::Mutant],
            2,
            1,
        )
    })
}

/// Rocksteady, Mutant Marauder — trample; a counter on a target creature as
/// another nontoken creature of yours enters.
pub fn rocksteady_mutant_marauder() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::PartnerWith("Bebop, Skull & Crossbones".into()), Keyword::Trample],
        triggered_abilities: vec![
            partner_with_search("Bebop, Skull & Crossbones"),
            on_yours_enters(
                EventScope::AnotherOfYours,
                R::Creature.and(R::NotToken),
                plus(target_filtered(R::Creature), Value::ONE),
            ),
        ],
        ..creature(
            "Rocksteady, Mutant Marauder",
            cost(&[generic(2), g()]),
            vec![CreatureType::Rhino, CreatureType::Mutant],
            3,
            3,
        )
    })
}

/// Big Mother Mouser — enters with two counters, doubles them attacking,
/// leaves that many Robots.
pub fn big_mother_mouser() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![
            on_attack(Effect::DoubleCountersOnEach { what: Selector::This, kind: CounterType::PlusOnePlusOne }),
            on_dies(create(
                robot_token(),
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
            )),
        ],
        ..creature("Big Mother Mouser", cost(&[generic(4)]), vec![CreatureType::Robot], 0, 0)
    }
}

/// Casey Jones, Back Alley Brute — a counter on an attacker; your +1/+1
/// counters on your creatures burn an opponent that much.
pub fn casey_jones_back_alley_brute() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            on_attack(plus(target_filtered(R::Creature.and(R::IsAttacking)), Value::ONE)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YouPutCounters)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: yours(R::Creature),
                    }),
                effect: Effect::DealDamage {
                    to: target_filtered(R::OpponentPlayer),
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..creature(
            "Casey Jones, Back Alley Brute",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            4,
            2,
        )
    })
}

/// Dimension X Pizzasaur — two counters, then destroys a creature no bigger
/// than your counters; sacrifices to drain 3.
pub fn dimension_x_pizzasaur() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food],
            creature_types: vec![CreatureType::Alien, CreatureType::Mutant],
            ..Default::default()
        },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            plus(target_filtered(R::Creature), Value::Const(2)),
            Effect::ReflexiveTrigger {
                body: Box::new(Effect::Destroy {
                    what: target_filtered(R::Creature.and(R::ManaValueAtMostCountersAmongYours)),
                }),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
            ]),
            ..Default::default()
        }],
        ..creature("Dimension X Pizzasaur", cost(&[generic(3), b()]), vec![], 2, 1)
    }
}

/// Electric Seaweed — defender, haste; as it enters, each death this turn
/// pings every non-Wall creature; taps to ping.
pub fn electric_seaweed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Haste],
        triggered_abilities: vec![etb(Effect::WheneverCreatureDiesThisTurn {
            filter: R::Creature.and(R::OtherThanSource),
            body: Box::new(Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Wall).negate())),
                amount: Value::ONE,
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Electric Seaweed",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Plant, CreatureType::Wall],
            0,
            4,
        )
    }
}

/// Irma, Part-Time Mutant — each combat on your turn may become a copy of
/// another creature of yours (keeping her name and this ability), then grows.
pub fn irma_part_time_mutant() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::BecomeCopyKeepingName {
                        what: Selector::This,
                        source: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
                        keywords: vec![],
                    },
                    plus(Selector::This, Value::ONE),
                ])),
            },
        }],
        ..creature(
            "Irma, Part-Time Mutant",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Mutant, CreatureType::Shapeshifter],
            1,
            1,
        )
    })
}

/// Krang, the All-Powerful — draw-caused triggers of yours fire twice; grows
/// on anyone's second draw.
pub fn krang_the_all_powerful() -> CardDefinition {
    legendary(CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "If a player drawing a card causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerDrawTriggers,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::AnyPlayer),
            effect: plus(Selector::This, Value::ONE),
        }],
        ..creature(
            "Krang, the All-Powerful",
            cost(&[generic(4), u()]),
            vec![CreatureType::Utrom, CreatureType::Robot],
            3,
            3,
        )
    })
}

/// Leatherhead, Iron Gator — two counters on each of your creatures as it
/// attacks.
pub fn leatherhead_iron_gator() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![on_attack(plus(Selector::EachPermanent(yours(R::Creature)), Value::Const(2)))],
        ..creature(
            "Leatherhead, Iron Gator",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Crocodile, CreatureType::Mutant, CreatureType::Rogue],
            5,
            5,
        )
    })
}

/// Mona Lisa, Science Geek — taps for mana equal to its power.
pub fn mona_lisa_science_geek() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..creature(
            "Mona Lisa, Science Geek",
            cost(&[generic(2), g()]),
            vec![CreatureType::Lizard, CreatureType::Mutant],
            1,
            3,
        )
    })
}

/// Rat King, Pale Piper — a Rat whenever it or another nontoken creature of
/// yours leaves; sacrifices tokens to draw.
pub fn rat_king_pale_piper() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_nontoken_leaves(R::Creature.and(R::NotToken), create(rat_token(), Value::ONE))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::IsToken, 1)),
            effect: draw(Value::ONE),
            ..Default::default()
        }],
        ..creature(
            "Rat King, Pale Piper",
            cost(&[generic(3), b()]),
            vec![CreatureType::Rat, CreatureType::Avatar],
            3,
            3,
        )
    })
}

/// Ray Fillet, Wave Warrior — flying, evolve; draws when a countered
/// creature of yours connects.
pub fn ray_fillet_wave_warrior() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            evolve(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .dealt_by(yours(R::Creature).and(R::WithAnyCounter)),
                effect: draw(Value::ONE),
            },
        ],
        ..creature(
            "Ray Fillet, Wave Warrior",
            cost(&[generic(2), u()]),
            vec![CreatureType::Fish, CreatureType::Mutant],
            0,
            2,
        )
    })
}

/// Shredder, Shadow Master — non-legendary copies attack each other
/// opponent (sacrificed at end of combat); a hit halves a life total.
pub fn shredder_shadow_master() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::CopiesAttackEachOtherOpponent {
                non_legendary: true,
                cleanup: AttackingTokenCleanup::SacrificeAtEndOfCombat,
            }),
            on_combat_damage_to_player(Effect::LoseLife {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::HalfLifeRoundedUp(PlayerRef::TriggerEventPlayer),
            }),
        ],
        ..creature(
            "Shredder, Shadow Master",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            5,
            5,
        )
    })
}

/// Tempestra, Dame of Games — sacrifices an artifact for a hasty
/// non-legendary copy of another creature of yours, gone at end step.
pub fn tempestra_dame_of_games() -> CardDefinition {
    legendary(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    extra_keywords: vec![],
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: true,
                    legendary: false,
                },
                Effect::GrantKeyword {
                    what: Selector::LastCreatedToken,
                    keyword: Keyword::Haste,
                    duration: Duration::Permanent,
                },
                Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedToken },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Tempestra, Dame of Games",
            cost(&[generic(1), r()]),
            vec![CreatureType::Elemental, CreatureType::Illusion],
            1,
            3,
        )
    })
}

/// Tokka & Rahzar, Unsupervised — once a turn, another nontoken creature of
/// yours leaving grows it and makes a Treasure.
pub fn tokka_rahzar_unsupervised() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken).and(R::OtherThanSource),
                })
                .once_per_turn(),
            effect: Effect::Seq(vec![plus(Selector::This, Value::ONE), create(treasure_token(), Value::ONE)]),
        }],
        ..creature(
            "Tokka & Rahzar, Unsupervised",
            cost(&[generic(2), r()]),
            vec![CreatureType::Turtle, CreatureType::Wolf, CreatureType::Mutant],
            3,
            3,
        )
    })
}

/// Vigor — damage to your other creatures becomes +1/+1 counters; shuffles
/// back from any graveyard trip.
///
/// ⚠ Residual: modelled as a replacement, so "damage can't be prevented"
/// doesn't stop it.
pub fn vigor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to another creature you control, prevent that damage. Put a +1/+1 counter on that creature for each 1 damage prevented this way.",
            effect: StaticEffect::ReplaceDamageToOtherCreaturesYouControlWithCounters {
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::ShuffleSelfIntoLibrary,
        }],
        ..creature(
            "Vigor",
            cost(&[generic(3), g(), g(), g()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            6,
            6,
        )
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Arcade Cabinet — counters on up to four creatures; sacrifices a token to
/// double a creature's counters.
pub fn arcade_cabinet() -> CardDefinition {
    CardDefinition {
        name: "Arcade Cabinet",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(plus(Selector::Target(0), Value::ONE)),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_other_filter: Some((R::IsToken, 1)),
            effect: Effect::DoubleAllCountersOn { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Coin of Mastery — a creature of yours enters with a counter per mana from
/// artifacts spent on it; taps for a Treasure.
///
/// ⚠ Residual: artifact mana is counted off the pool, so mana floated from
/// lands and artifacts together and only partly spent can read low.
pub fn coin_of_mastery() -> CardDefinition {
    CardDefinition {
        name: "Coin of Mastery",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control enters with an additional +1/+1 counter on it for each mana from an artifact source spent to cast it.",
            effect: StaticEffect::ExtraEtbCountersForCreatureCasts {
                kind: CounterType::PlusOnePlusOne,
                value: Value::ArtifactManaSpentToCastSource,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: create(treasure_token(), Value::ONE),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Everything Pizza — fetches a basic; everything at once for WUBRG.
pub fn everything_pizza() -> CardDefinition {
    CardDefinition {
        name: "Everything Pizza",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), u(), b(), r(), g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::OptionalTargets {
                min: 2,
                body: Box::new(Effect::Seq(vec![
                    Effect::GainLife { who: target_filtered(R::Player), amount: Value::Const(3) },
                    Effect::Draw { who: Selector::Target(0), amount: Value::ONE },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::ONE,
                        random: false,
                    },
                    Effect::DealDamage {
                        to: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::Creature.or(R::Player).or(R::Planeswalker),
                        },
                        amount: Value::Const(3),
                    },
                    plus(Selector::TargetFiltered { slot: 2, filter: R::Creature }, Value::Const(3)),
                ])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Exploding Barrel — any color, one pressure counter a tap; blows up a
/// creature for 20, a generic cheaper per pressure counter.
pub fn exploding_barrel() -> CardDefinition {
    CardDefinition {
        name: "Exploding Barrel",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Pressure, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(8)]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                self_counter_cost_reduction: Some(CounterType::Pressure),
                effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(20) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Foot Chopper — comes with a Ninja; the flier may be sacrificed to draw its
/// power after a hit.
pub fn foot_chopper() -> CardDefinition {
    CardDefinition {
        name: "Foot Chopper",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            create(ninja_token(), Value::ONE),
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying],
            triggered_abilities: vec![on_combat_damage_to_player(may(
                "Sacrifice this creature to draw cards equal to its power?",
                Effect::Seq(vec![
                    draw(Value::PowerOf(Box::new(Selector::This))),
                    Effect::SacrificePermanent { what: Selector::This },
                ]),
            ))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Mole Module — a 6/6 menace Vehicle; a hit mills four and may put a
/// permanent card among them onto the battlefield.
pub fn mole_module() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        keywords: vec![Keyword::Menace, Keyword::Crew(2)],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(4) },
            Effect::MoveChosen {
                from: Selector::LastMoved,
                filter: Some(R::PermanentCard),
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]))],
        ..creature("Mole Module", cost(&[generic(5)]), vec![], 6, 6)
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Endless Foot Assault — squad; each attack sends a Ninja at every opponent.
pub fn endless_foot_assault() -> CardDefinition {
    CardDefinition {
        name: "Endless Foot Assault",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Squad(cost(&[generic(1), w()]))],
        triggered_abilities: vec![
            squad_etb(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::ForEachOpponent {
                    body: Box::new(Effect::CreateTokenAttacking {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(ninja_token()),
                        cleanup: AttackingTokenCleanup::None,
                        defender: Some(PlayerRef::Triggerer),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// High Score — one more +1/+1 counter each time; draws at your end step
/// while you have the biggest creature.
pub fn high_score() -> CardDefinition {
    CardDefinition {
        name: "High Score",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCounters,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ControlsGreatestPowerCreature { who: PlayerRef::You },
                then: Box::new(draw(Value::ONE)),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Level Up — a counter as it lands; the creature doubles its counters
/// attacking and draws at power 10.
pub fn level_up() -> CardDefinition {
    CardDefinition {
        name: "Level Up",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(plus(Selector::AttachedTo(Box::new(Selector::This)), Value::ONE))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(Effect::Seq(vec![
                Effect::DoubleCountersOnEach { what: Selector::This, kind: CounterType::PlusOnePlusOne },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::PowerOf(Box::new(Selector::This)), Value::Const(10)),
                    then: Box::new(draw(Value::ONE)),
                    else_: Box::new(Effect::Noop),
                },
            ]))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ninja Pizza — your Foods tap for mana; a Food each second main phase.
pub fn ninja_pizza() -> CardDefinition {
    CardDefinition {
        name: "Ninja Pizza",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Foods you control have \"{T}, Sacrifice this artifact: Add one mana of any color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(yours(R::HasArtifactSubtype(ArtifactSubtype::Food))),
                ability: ActivatedAbility {
                    tap_cost: true,
                    sac_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        triggered_abilities: vec![your_second_main(create(food_token(), Value::ONE))],
        ..Default::default()
    }
}

// ── Instants and sorceries ──────────────────────────────────────────────────

/// Continue? — up to four creature cards of yours that died this turn come
/// back.
pub fn continue_card() -> CardDefinition {
    spell(
        "Continue?",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Creature.and(R::InGraveyard).and(R::OwnedByYou).and(R::PutIntoGraveyardFromBattlefieldThisTurn),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
    )
}

/// Double Jump // Flying Kick — a flying counter and base 5/5; or a bite.
///
/// ⚠ Residual: cast fused, Flying Kick has no enemy target (a fused right
/// half reads one target), so only Double Jump does anything.
pub fn double_jump_flying_kick() -> CardDefinition {
    let mine = target_filtered(yours(R::Creature));
    CardDefinition {
        name: "Double Jump // Flying Kick",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddKeywordCounter { what: mine, keyword: Keyword::Flying, amount: Value::ONE },
            Effect::SetBasePT {
                what: Selector::Target(0),
                power: Value::Const(5),
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf { cost: cost(&[generic(1), r()]), card_types: vec![CardType::Instant], effect: bite() },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Fast Forward — cheaper per opponent you attacked; goads every opposing
/// creature.
pub fn fast_forward() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each opponent you attacked this turn.",
            effect: StaticEffect::SelfCostReducedByValue { amount: Value::OpponentsAttackedThisTurn },
        }],
        ..spell(
            "Fast Forward",
            cost(&[generic(4), r()]),
            CardType::Sorcery,
            Effect::Goad { what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)) },
        )
    }
}

/// Game Over — a wrath, {2} cheaper once anyone is at half life.
pub fn game_over() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if a player's life total is less than or equal to half their starting life total.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::ValueAtLeast(
                    Value::StartingLifeTotal,
                    Value::Times(Box::new(Value::LowestLifeTotal), Box::new(Value::Const(2))),
                ),
                amount: 2,
            },
        }],
        ..spell(
            "Game Over",
            cost(&[generic(3), b(), b()]),
            CardType::Sorcery,
            Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
        )
    }
}

/// Here Comes a New Hero! — a player draws X; copy a creature of mana value
/// X or less.
pub fn here_comes_a_new_hero() -> CardDefinition {
    spell(
        "Here Comes a New Hero!",
        cost(&[crate::mana::x(), generic(2), u()]),
        CardType::Sorcery,
        Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::Draw { who: target_filtered(R::Player), amount: Value::XFromCost },
                Effect::CreateTokenCopyOf {
                    extra_keywords: vec![],
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ManaValueAtMostXFromCost),
                    },
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                },
            ])),
        },
    )
}

/// Lessons from Life — draw three, then a land from hand tapped.
pub fn lessons_from_life() -> CardDefinition {
    spell(
        "Lessons from Life",
        cost(&[generic(2), g(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            draw(Value::Const(3)),
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Land,
                count: Value::ONE,
                tapped: true,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        ]),
    )
}

/// Shellshock — X to one creature per opponent; a Mutagen per creature hit.
pub fn shellshock() -> CardDefinition {
    spell(
        "Shellshock",
        cost(&[crate::mana::x(), r()]),
        CardType::Instant,
        Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::Target(0), amount: Value::XFromCost },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(Value::XFromCost, Value::ONE),
                        then: Box::new(create(mutagen_token(), Value::ONE)),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            }),
        },
    )
}

/// Special Move — choose two: destroy an artifact; two counters on a
/// combatant of yours; fling a creature.
///
/// ⚠ Residual: Foot Toss's creature is your greatest-power creature, chosen
/// on resolution rather than targeted (a mode binds one target slot).
pub fn special_move() -> CardDefinition {
    spell(
        "Special Move",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                plus(target_filtered(yours(R::Creature).and(R::IsAttacking.or(R::IsBlocking))), Value::Const(2)),
                Effect::Seq(vec![
                    Effect::DealDamageEqualToPower { source: Selector::GreatestPowerYouControl, target: target_any() },
                    Effect::SacrificePermanent { what: Selector::GreatestPowerYouControl },
                ]),
            ],
            min: 2,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Super Combo — replicate {2}; a creature of yours bites an opponent's.
pub fn super_combo() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Replicate(cost(&[generic(2)]))],
        ..spell("Super Combo", cost(&[generic(1), g()]), CardType::Sorcery, bite())
    }
}

/// Swift Demise — 1 damage to a creature, then every damaged creature you
/// don't control dies.
pub fn swift_demise() -> CardDefinition {
    spell(
        "Swift Demise",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
            Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou.negate()).and(R::DealtDamageThisTurn),
                ),
            },
        ]),
    )
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// Big Apple, 3 a.m. — enters tapped naming a color; a Rat per opponent for
/// {5}.
pub fn big_apple_3_a_m() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enters_tapped()],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: create(rat_token(), Value::OpponentCount),
                ..Default::default()
            },
        ],
        ..land("Big Apple, 3 a.m.")
    }
}

/// Hidden Hideout — tapped Command Tower; lifelink for a countered creature.
pub fn hidden_hideout() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add_commander_identity(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: keyword_eot(target_filtered(yours(R::Creature).and(R::WithAnyCounter)), Keyword::Lifelink),
                ..Default::default()
            },
        ],
        ..land("Hidden Hideout")
    }
}

/// Turtle Lair — {C}, or any color for Ninja and Turtle spells; makes a Ninja
/// or Turtle unblockable.
pub fn turtle_lair() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::CreatureOfAnyTypes([
                            CreatureType::Ninja,
                            CreatureType::Turtle,
                            CreatureType::Turtle,
                        ]),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: keyword_eot(
                    target_filtered(R::HasCreatureType(CreatureType::Ninja).or(R::HasCreatureType(CreatureType::Turtle))),
                    Keyword::Unblockable,
                ),
                ..Default::default()
            },
        ],
        ..land("Turtle Lair")
    }
}
