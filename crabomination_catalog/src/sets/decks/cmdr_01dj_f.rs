//! Commander: 2026-10-19 (`01DJHkWk`) — closers for the Villain, Goblin and
//! Runaways decks the earlier batches left one or two cards short. Tests in
//! `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{connive, etb, on_attack, on_dies, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, hybrid, r, u};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..def
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

fn villain() -> R {
    R::HasCreatureType(CreatureType::Villain)
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn illusion_villain() -> TokenDefinition {
    TokenDefinition {
        name: "Illusion Villain".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Illusion, CreatureType::Villain],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn basic_landcycling() -> Keyword {
    Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))
}

/// Bolg of the North — {3}{B}{R} legendary 5/5 Goblin Soldier. Enters: you
/// may sacrifice another creature; when you do (CR 603.12), Bolg deals damage
/// equal to its last-known power to another target creature, and excess
/// damage amasses Goblins that much.
pub fn bolg_of_the_north() -> CardDefinition {
    let excess = || Value::ExcessDamageDealtThisResolution;
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice another creature for Bolg?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature.and(R::OtherThanSource),
                },
                Effect::If {
                    cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                    then: Box::new(Effect::ReflexiveTrigger { body: Box::new(Effect::Seq(vec![
                        Effect::DealDamage {
                            to: target_filtered(R::Creature.and(R::OtherThanSource)),
                            amount: Value::SacrificedPower,
                        },
                        Effect::If {
                            cond: Predicate::ValueAtLeast(excess(), Value::ONE),
                            then: Box::new(Effect::Amass {
                                who: PlayerRef::You,
                                count: excess(),
                                extra_type: Some(CreatureType::Goblin),
                            }),
                            else_: Box::new(Effect::Noop),
                        },
                    ])) }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..legend(creature(
            "Bolg of the North",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Goblin, CreatureType::Soldier],
            5,
            5,
        ))
    }
}

/// The Balrog, Flame of Udûn — {3}{B}{R} legendary 7/7 trample Avatar Demon.
/// When a legendary creature an opponent controls dies, it goes to the bottom
/// of its owner's library.
pub fn the_balrog_flame_of_udun() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasSupertype(Supertype::Legendary),
                }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                    pos: LibraryPosition::Bottom,
                },
            },
        }],
        ..legend(creature(
            "The Balrog, Flame of Udûn",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Avatar, CreatureType::Demon],
            7,
            7,
        ))
    }
}

/// Void Helix — {4}{B}{B} instant. 5 damage to any target; you gain 5 life.
pub fn void_helix() -> CardDefinition {
    spell(
        "Void Helix",
        cost(&[generic(4), b(), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::any_target()),
                amount: Value::Const(5),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(5),
            },
        ]),
    )
}

/// Sinister Hideout — Land. Enters tapped. {T}: {U} or {B}. {4}, {T}: surveil 1.
pub fn sinister_hideout() -> CardDefinition {
    let mut d =
        crate::sets::dual_land_untyped("Sinister Hideout", Color::Blue, Color::Black, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d.activated_abilities.push(crate::card::ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(4)]),
        effect: Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::ONE,
        },
        ..Default::default()
    });
    d
}

/// Vulture, Scheming Scavenger — {5}{U/B} legendary 4/6 flying Human
/// Artificer Villain. Attacking, other Villains you control gain flying until
/// end of turn.
pub fn vulture_scheming_scavenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: Selector::EachPermanent(mine().and(villain()).and(R::OtherThanSource)),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..legend(creature(
            "Vulture, Scheming Scavenger",
            cost(&[generic(5), hybrid(Color::Blue, Color::Black)]),
            vec![
                CreatureType::Human,
                CreatureType::Artificer,
                CreatureType::Villain,
            ],
            4,
            6,
        ))
    }
}

/// Living Lies of Loki — {1}{U} 1/3 Illusion Villain; +1/+0 for each other
/// Illusion you control; dying, draw a card.
pub fn living_lies_of_loki() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 for each other Illusion you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::HasCreatureType(CreatureType::Illusion).and(R::OtherThanSource),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Living Lies of Loki",
            cost(&[generic(1), u()]),
            vec![CreatureType::Illusion, CreatureType::Villain],
            1,
            3,
        )
    }
}

/// Mysterio's Mirage — {2}{U} enchantment. At the beginning of your end step,
/// if you discarded a card this turn, a 3/3 blue Illusion Villain.
pub fn mysterios_mirage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::DiscardedThisTurn {
                who: PlayerRef::You,
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(illusion_villain()),
            },
        }],
        ..spell(
            "Mysterio's Mirage",
            cost(&[generic(2), u()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Pharaoh Rama-Tut — {4}{U} legendary 4/4 Human Noble Villain, ward {2};
/// connives whenever you cast a noncreature spell.
pub fn pharaoh_rama_tut() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature.negate())),
            effect: connive(1),
        }],
        ..legend(creature(
            "Pharaoh Rama-Tut",
            cost(&[generic(4), u()]),
            vec![
                CreatureType::Human,
                CreatureType::Noble,
                CreatureType::Villain,
            ],
            4,
            4,
        ))
    }
}

/// Psionic Snoop — {2}{U} flash 0/3 Human Rogue; connives as it enters.
pub fn psionic_snoop() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(connive(1))],
        ..creature(
            "Psionic Snoop",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            0,
            3,
        )
    }
}

/// Atlas, Sizable Stooge — {2}{B/G}{B/G} legendary 5/4 reach Human Villain.
/// Attacking or blocking, gain 1 life per creature you control with power 4+.
pub fn atlas_sizable_stooge() -> CardDefinition {
    let gain = || Effect::GainLife {
        who: Selector::You,
        amount: Value::count(Selector::EachPermanent(mine().and(R::PowerAtLeast(4)))),
    };
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            on_attack(gain()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: gain(),
            },
        ],
        ..legend(creature(
            "Atlas, Sizable Stooge",
            cost(&[generic(2), bg(), bg()]),
            vec![CreatureType::Human, CreatureType::Villain],
            5,
            4,
        ))
    }
}

/// Colossal Collision — {3}{G} sorcery. A +1/+1 counter on target creature
/// you control, then it deals damage equal to its power to target creature an
/// opponent controls. Basic landcycling {2}.
pub fn colossal_collision() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling()],
        ..spell(
            "Colossal Collision",
            cost(&[generic(3), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(mine()),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::DealDamageEqualToPower {
                    source: Selector::Target(0),
                    target: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                },
            ]),
        )
    }
}

/// Gnashing of Teeth — {1}{B}{B} sorcery. Choose one: target creature gets
/// -5/-5, exiled if it would die this turn; or creatures target player
/// controls get -1/-1 until end of turn.
pub fn gnashing_of_teeth() -> CardDefinition {
    let shrink = |what, n: i32| Effect::PumpPT {
        what,
        power: Value::Const(-n),
        toughness: Value::Const(-n),
        duration: Duration::EndOfTurn,
    };
    spell(
        "Gnashing of Teeth",
        cost(&[generic(1), b(), b()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::ExileIfWouldDieThisTurn {
                    what: target_filtered(R::Creature),
                },
                shrink(Selector::Target(0), 5),
            ]),
            Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(shrink(
                    Selector::ControlledBy {
                        who: PlayerRef::Target(0),
                        filter: R::Creature,
                    },
                    1,
                )),
            },
        ]),
    )
}

/// Crimson Operative — {3}{R} 3/2 prowess Human Villain artifact creature.
/// Enters: exile your top card; you may play it until the end of your next
/// turn.
pub fn crimson_operative() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        ..creature(
            "Crimson Operative",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Villain],
            3,
            2,
        )
    }
}

/// Pinecone Strike — {1}{R} instant. Choose one or both: 3 damage to target
/// creature, exiled if it would die this turn; destroy target artifact token.
pub fn pinecone_strike() -> CardDefinition {
    spell(
        "Pinecone Strike",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Seq(vec![
                    Effect::ExileIfWouldDieThisTurn {
                        what: target_filtered(R::Creature),
                    },
                    Effect::DealDamage {
                        to: Selector::Target(0),
                        amount: Value::Const(3),
                    },
                ]),
                Effect::Destroy {
                    what: target_filtered(R::Artifact.and(R::IsToken)),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Merciless Resolve — {2}{B} instant. As an additional cost, sacrifice a
/// creature or land. Draw two cards.
pub fn merciless_resolve() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.or(R::Land),
            count: 1,
        }],
        ..spell(
            "Merciless Resolve",
            cost(&[generic(2), b()]),
            CardType::Instant,
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        )
    }
}

/// Painful Lesson — {2}{B} sorcery. Target player draws two and loses 2 life.
pub fn painful_lesson() -> CardDefinition {
    spell(
        "Painful Lesson",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw {
                who: target_filtered(R::Player),
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Karolina Dean, Runaway — {3}{R} legendary 3/3 flying Alien Hero. At the
/// beginning of your first main phase, add {W}{U}{B}{R}{G} that can't be spent
/// on spells cast from your hand.
pub fn karolina_dean_runaway() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![
                        Color::White,
                        Color::Blue,
                        Color::Black,
                        Color::Red,
                        Color::Green,
                    ])),
                    SpendRestriction::NotSpellsFromHand,
                ),
            },
        }],
        ..legend(creature(
            "Karolina Dean, Runaway",
            cost(&[generic(3), r()]),
            vec![CreatureType::Alien, CreatureType::Hero],
            3,
            3,
        ))
    }
}

/// Gert and Old Lace, Runaways — {2}{R/G} legendary 4/3 trample Human
/// Dinosaur Hero. Enters: you may discard a card; if you do, search for a
/// basic land card to hand.
pub fn gert_and_old_lace_runaways() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Discard a card to search for a basic land?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::DiscardedThisResolution {
                        filter: R::Any,
                    }),
                    then: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: R::IsBasicLand,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..legend(creature(
            "Gert and Old Lace, Runaways",
            cost(&[generic(2), hybrid(Color::Red, Color::Green)]),
            vec![
                CreatureType::Human,
                CreatureType::Dinosaur,
                CreatureType::Hero,
            ],
            4,
            3,
        ))
    }
}
