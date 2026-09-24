//! Commander: the cards the **Raining Cats and Dogs** Secret Lair Commander
//! deck (SLD, Rin and Seri, Inseparable) needed beyond what the catalog had.
//! Tests in `tests/recent_b/cmdr_rinseri.rs`.
//!
//! Residuals (each also on its card):
//! - **Highcliff Felidar** — the per-opponent destructions happen one
//!   opponent at a time, not simultaneously; the pick among tied creatures is
//!   the engine's, not the controller's.
//! - **Jinnie Fay** — the replacement is not optional: a creature token whose
//!   printed body is smaller than a Cat's or Dog's always becomes the bigger
//!   one, and a noncreature token (Treasure, Clue) is never replaced.
//! - **Pack Leader** — the shield covers the Dogs you control as the trigger
//!   resolves, not a Dog that arrives later in the turn.
//! - **Showdown of the Skalds** — chapters II and III choose the counter's
//!   target as each trigger resolves.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, g, generic, hybrid, r, w, x};
use crate::sets::tap_add_colorless;
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn creature_token(
    name: &str,
    ct: CreatureType,
    colors: Vec<Color>,
    p: i32,
    t: i32,
    keywords: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn citizen() -> Arc<TokenDefinition> {
    Arc::new(creature_token("Citizen", CreatureType::Citizen, vec![Color::Green, Color::White], 1, 1, vec![]))
}

fn rgw() -> crate::mana::ManaCost {
    cost(&[generic(1), r(), g(), w()])
}

/// Rin and Seri, Inseparable — casting a Dog spell makes a 1/1 Cat, casting
/// a Cat spell makes a 1/1 Dog; {R}{G}{W}, {T}: damage equal to your Dogs to
/// any target, and gain life equal to your Cats.
pub fn rin_and_seri_inseparable() -> CardDefinition {
    let on_cast = |ct: CreatureType, token: TokenDefinition| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::HasCreatureType(ct))),
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) },
    };
    CardDefinition {
        triggered_abilities: vec![
            on_cast(CreatureType::Dog, creature_token("Cat", CreatureType::Cat, vec![Color::Green], 1, 1, vec![])),
            on_cast(CreatureType::Cat, creature_token("Dog", CreatureType::Dog, vec![Color::White], 1, 1, vec![])),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), g(), w()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_any(),
                    amount: Value::count(yours(R::Creature.and(R::HasCreatureType(CreatureType::Dog)))),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::count(yours(R::Creature.and(R::HasCreatureType(CreatureType::Cat)))),
                },
            ]),
            ..Default::default()
        }],
        ..legendary(creature(
            "Rin and Seri, Inseparable",
            rgw(),
            vec![CreatureType::Dog, CreatureType::Cat],
            4,
            4,
        ))
    }
}

/// Jetmir, Nexus of Revels — your creatures get +1/+0 and vigilance at three
/// creatures, another +1/+0 and trample at six, another +1/+0 and double
/// strike at nine.
pub fn jetmir_nexus_of_revels() -> CardDefinition {
    let tier = |n: i32, keyword: Keyword, description: &'static str| {
        let at_least = || Predicate::SelectorCountAtLeast { sel: yours(R::Creature), n: Value::Const(n) };
        [
            StaticAbility {
                description,
                effect: StaticEffect::WhileCondition {
                    condition: at_least(),
                    inner: Box::new(StaticEffect::PumpPT { applies_to: yours(R::Creature), power: 1, toughness: 0 }),
                },
            },
            StaticAbility {
                description,
                effect: StaticEffect::WhileCondition {
                    condition: at_least(),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: yours(R::Creature), keyword }),
                },
            },
        ]
    };
    let mut statics = Vec::new();
    statics.extend(tier(3, Keyword::Vigilance, "Three or more creatures: +1/+0 and vigilance."));
    statics.extend(tier(6, Keyword::Trample, "Six or more creatures: another +1/+0 and trample."));
    statics.extend(tier(9, Keyword::DoubleStrike, "Nine or more creatures: another +1/+0 and double strike."));
    CardDefinition {
        static_abilities: statics,
        ..legendary(creature(
            "Jetmir, Nexus of Revels",
            rgw(),
            vec![CreatureType::Cat, CreatureType::Demon],
            5,
            4,
        ))
    }
}

/// Jinnie Fay, Jetmir's Second — your tokens may instead be 2/2 green Cats
/// with haste or 3/1 green Dogs with vigilance (CR 614.1a).
///
/// ⚠ Residual: only creature tokens are replaced, and only by a bigger body
/// (see `GameState::token_replacement_for`).
pub fn jinnie_fay_jetmirs_second() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your tokens may instead be 2/2 Cats with haste or 3/1 Dogs with vigilance.",
            effect: StaticEffect::TokensMayBecome {
                options: vec![
                    creature_token("Cat", CreatureType::Cat, vec![Color::Green], 2, 2, vec![Keyword::Haste]),
                    creature_token("Dog", CreatureType::Dog, vec![Color::Green], 3, 1, vec![Keyword::Vigilance]),
                ],
            },
        }],
        ..legendary(creature(
            "Jinnie Fay, Jetmir's Second",
            cost(&[hybrid(Color::Red, Color::Green), g(), hybrid(Color::Green, Color::White)]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            3,
        ))
    }
}

/// Kitt Kanto, Mayhem Diva — a 1/1 Citizen on entry; at the beginning of
/// combat on each player's turn, you may tap two untapped creatures you
/// control; when you do, a creature that player controls gets +2/+2 and
/// trample until end of turn and is goaded (CR 701.15).
pub fn kitt_kanto_mayhem_diva() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: citizen() }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
                effect: Effect::MayTap {
                    description: "Tap two untapped creatures to pump and goad a creature?".into(),
                    filter: R::Creature.and(R::ControlledByYou),
                    count: Value::Const(2),
                    then: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::Seq(vec![
                            Effect::PumpPT {
                                what: target_filtered(R::Creature.and(R::ControlledByActivePlayer)),
                                power: Value::Const(2),
                                toughness: Value::Const(2),
                                duration: Duration::EndOfTurn,
                            },
                            Effect::GrantKeyword {
                                what: Selector::Target(0),
                                keyword: Keyword::Trample,
                                duration: Duration::EndOfTurn,
                            },
                            Effect::Goad { what: Selector::Target(0) },
                        ])),
                    }),
                    else_: None,
                },
            },
        ],
        ..legendary(creature(
            "Kitt Kanto, Mayhem Diva",
            rgw(),
            vec![CreatureType::Cat, CreatureType::Bard, CreatureType::Druid],
            3,
            3,
        ))
    }
}

/// Phabine, Boss's Confidant — creature tokens you control have haste.
/// Parley at the beginning of combat on your turn: a Citizen per land
/// revealed, then +1/+1 to your creatures per nonland revealed, then each
/// player draws.
pub fn phabine_bosss_confidant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::IsToken)),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Parley {
                then: Box::new(Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::LandCardsRevealedThisEffect,
                        definition: citizen(),
                    },
                    Effect::PumpPT {
                        what: yours(R::Creature),
                        power: Value::CardsRevealedThisEffect,
                        toughness: Value::CardsRevealedThisEffect,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        }],
        ..legendary(creature(
            "Phabine, Boss's Confidant",
            cost(&[generic(3), r(), g(), w()]),
            vec![CreatureType::Cat, CreatureType::Advisor],
            3,
            6,
        ))
    }
}

/// Animal Sanctuary — {T}: add {C}; {2}, {T}: a +1/+1 counter on target
/// Bird, Cat, Dog, Goat, Ox, or Snake.
pub fn animal_sanctuary() -> CardDefinition {
    let animal = [
        CreatureType::Bird,
        CreatureType::Cat,
        CreatureType::Dog,
        CreatureType::Goat,
        CreatureType::Ox,
        CreatureType::Snake,
    ]
    .into_iter()
    .map(R::HasCreatureType)
    .reduce(R::or)
    .expect("six types");
    CardDefinition {
        name: "Animal Sanctuary",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(animal)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Bloodline Pretender — changeling; as it enters, choose a creature type;
/// another creature of yours of that type entering puts a +1/+1 counter on it.
pub fn bloodline_pretender() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Changeling],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                    Predicate::TriggerObjectIsChosenType,
                ])),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Bloodline Pretender", cost(&[generic(3)]), vec![CreatureType::Shapeshifter], 2, 2)
    }
}

fn cats_you_control() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Cat)).and(R::ControlledByYou)
}

/// Feline Sovereign — other Cats you control get +1/+1 and have protection
/// from Dogs; whenever one or more Cats you control deal combat damage to a
/// player, destroy up to one target artifact or enchantment that player
/// controls.
pub fn feline_sovereign() -> CardDefinition {
    let others = || Selector::EachPermanent(cats_you_control().and(R::OtherThanSource));
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other Cats you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: others(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Other Cats you control have protection from Dogs.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: others(),
                    keyword: Keyword::ProtectionFromCreatureType(CreatureType::Dog),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Cat),
                    },
                )
            },
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Artifact.or(R::Enchantment).and(R::ControlledByTriggerPlayer),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
        }],
        ..creature("Feline Sovereign", cost(&[generic(2), g()]), vec![CreatureType::Cat], 2, 3)
    }
}

/// Greater Tanuki — trample; channel — {2}{G}, discard it: a basic land onto
/// the battlefield tapped.
pub fn greater_tanuki() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature("Greater Tanuki", cost(&[generic(4), g(), g()]), vec![CreatureType::Dog], 6, 5)
    }
}

/// Highcliff Felidar — vigilance; on entry, destroy a greatest-power creature
/// of each opponent's.
///
/// ⚠ Residual: the destructions run one opponent at a time, and the engine
/// picks among tied creatures.
pub fn highcliff_felidar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::EachPlayerDoes {
            who: PlayerRef::EachOpponent,
            body: Box::new(Effect::Destroy {
                what: Selector::TakeGreatestPower {
                    inner: Box::new(yours(R::Creature)),
                    count: Box::new(Value::ONE),
                },
            }),
        })],
        ..creature(
            "Highcliff Felidar",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Cat, CreatureType::Beast],
            5,
            5,
        )
    }
}

/// Komainu Battle Armor — menace; equipped creature gets +2/+2 and has
/// menace; whenever it or the equipped creature deals combat damage to a
/// player, goad each creature that player controls. Reconfigure {4}.
pub fn komainu_battle_armor() -> CardDefinition {
    let goad_them = || TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::Goad { what: Selector::EachPermanent(R::Creature.and(R::ControlledByTriggerPlayer)) },
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        keywords: vec![Keyword::Menace, Keyword::Reconfigure(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Menace],
            triggered_abilities: vec![goad_them()],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        triggered_abilities: vec![goad_them()],
        ..creature("Komainu Battle Armor", cost(&[generic(2), r()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Mirror Entity — changeling; {X}: until end of turn, your creatures have
/// base power and toughness X/X and gain all creature types.
pub fn mirror_entity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            effect: Effect::Seq(vec![
                Effect::SetBasePT {
                    what: yours(R::Creature),
                    power: Value::XFromCost,
                    toughness: Value::XFromCost,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: yours(R::Creature),
                    keyword: Keyword::Changeling,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Mirror Entity", cost(&[generic(2), w()]), vec![CreatureType::Shapeshifter], 1, 1)
    }
}

/// Nacatl War-Pride — must be blocked by exactly one creature if able;
/// whenever it attacks, one tapped and attacking copy per creature the
/// defending player controls, exiled at the next end step.
pub fn nacatl_war_pride() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustBeBlocked, Keyword::CantBeBlockedByMoreThanOne],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByDefendingPlayer))),
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: true,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
            Effect::ExileAtNextEndStep { what: Selector::LastCreatedTokens },
        ]))],
        ..creature(
            "Nacatl War-Pride",
            cost(&[generic(3), g(), g(), g()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Pack Leader — other Dogs you control get +1/+1; whenever it attacks,
/// prevent all combat damage that would be dealt this turn to your Dogs.
///
/// ⚠ Residual: the shield covers the Dogs you control as the trigger resolves.
pub fn pack_leader() -> CardDefinition {
    let dogs = || R::Creature.and(R::HasCreatureType(CreatureType::Dog));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Dogs you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: yours(dogs().and(R::OtherThanSource)), power: 1, toughness: 1 },
        }],
        triggered_abilities: vec![on_attack(Effect::PreventCombatDamageToTargetThisTurn { target: yours(dogs()) })],
        ..creature("Pack Leader", cost(&[generic(1), w()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Showdown of the Skalds — Saga. I: exile the top four; you may play them
/// until the end of your next turn. II, III: whenever you cast a spell this
/// turn, a +1/+1 counter on target creature you control.
///
/// ⚠ Residual: the counter's target is chosen as each trigger resolves.
pub fn showdown_of_the_skalds() -> CardDefinition {
    let counters = || Effect::OnEachSpellCastThisTurn {
        body: Box::new(Effect::Reflexive {
            body: Box::new(Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        }),
    };
    CardDefinition {
        name: "Showdown of the Skalds",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            ),
            (2, counters()),
            (3, counters()),
        ],
        ..Default::default()
    }
}

/// Skyhunter Strike Force — flying, melee; lieutenant — while you control
/// your commander, your other creatures have melee.
pub fn skyhunter_strike_force() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Melee],
        static_abilities: vec![StaticAbility {
            description: "Lieutenant — other creatures you control have melee.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: yours(R::Creature.and(R::OtherThanSource)),
                    keyword: Keyword::Melee,
                }),
            },
        }],
        ..creature(
            "Skyhunter Strike Force",
            cost(&[generic(2), w()]),
            vec![CreatureType::Cat, CreatureType::Knight],
            2,
            2,
        )
    }
}
