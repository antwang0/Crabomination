//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan, fifth
//! batch. Tests in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{choose_one_then, chosen_one, etb, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, u, w};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn plus_one(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn assassin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Assassin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Menace],
        subtypes: Subtypes { creature_types: vec![CreatureType::Assassin], ..Default::default() },
        ..Default::default()
    }
}

fn shrines() -> Value {
    Value::count(yours(R::HasEnchantmentSubtype(EnchantmentSubtype::Shrine)))
}

/// Ezio, Brash Novice — {1}{R/W} legendary 1/1 Human. Attacking: a +1/+1
/// counter on it. With two or more counters it has first strike and is an
/// Assassin too. (Bayek of Siwa, Aya of Alexandria.)
pub fn ezio_brash_novice() -> CardDefinition {
    let two_counters = || Predicate::ValueAtLeast(Value::TotalCountersOn { what: Box::new(Selector::This) }, Value::Const(2));
    CardDefinition {
        triggered_abilities: vec![on_attack(plus_one(Selector::This, Value::ONE))],
        static_abilities: vec![
            StaticAbility {
                description: "As long as Ezio has two or more counters on it, it has first strike.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate { keyword: Keyword::FirstStrike, condition: two_counters() },
            },
            StaticAbility {
                description: "As long as Ezio has two or more counters on it, it is an Assassin in addition to its other types.",
                effect: StaticEffect::WhileCondition {
                    condition: two_counters(),
                    inner: Box::new(StaticEffect::AddCreatureTypeToMatching {
                        applies_to: Selector::This,
                        creature_type: CreatureType::Assassin,
                    }),
                },
            },
        ],
        ..legend("Ezio, Brash Novice", cost(&[generic(1), hybrid(Color::Red, Color::White)]), vec![CreatureType::Human], 1, 1)
    }
}

/// Origin of the Hidden Ones — {3}{R} Saga. I: 4 damage to any target. II:
/// two 1/1 black Assassins with menace. III: this turn, whenever an Assassin
/// you control attacks, a tapped and attacking Assassin. (Bayek, Aya.)
pub fn origin_of_the_hidden_ones() -> CardDefinition {
    CardDefinition {
        name: "Origin of the Hidden Ones",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::DealDamage { to: target_any(), amount: Value::Const(4) }),
            (2, Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(assassin_token()) }),
            (
                3,
                Effect::TriggersThisTurn {
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::HasCreatureType(CreatureType::Assassin),
                        }),
                        effect: Effect::CreateTokenAttacking {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Arc::new(assassin_token()),
                            cleanup: Default::default(),
                            defender: None,
                        },
                    }],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Thanos, Death's Consort — {2}{B}{B} legendary 3/3 Eternal Villain;
/// lifelink; whenever another creature dies, a +1/+1 counter on him.
pub fn thanos_deaths_consort() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::OtherThanSource,
            }),
            effect: plus_one(Selector::This, Value::ONE),
        }],
        ..legend("Thanos, Death's Consort", cost(&[generic(2), b(), b()]), vec![CreatureType::Eternal, CreatureType::Villain], 3, 3)
    }
}

/// Visions of Villainy — {2}{B} instant, {1} less with a Villain: draw two,
/// lose 2 life. (Thanos, Morbius.)
pub fn visions_of_villainy() -> CardDefinition {
    CardDefinition {
        name: "Visions of Villainy",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast if you control a Villain.",
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 1,
                condition: Predicate::SelectorCountAtLeast {
                    sel: yours(R::HasCreatureType(CreatureType::Villain)),
                    n: Value::ONE,
                },
            },
        }],
        effect: Effect::Seq(vec![draw(2), Effect::LoseLife { who: Selector::You, amount: Value::Const(2) }]),
        ..Default::default()
    }
}

/// Villainous Wrath — {3}{B}{B} sorcery: target opponent loses life equal to
/// the creatures they control, then destroy all creatures. (Morbius.)
pub fn villainous_wrath() -> CardDefinition {
    CardDefinition {
        name: "Villainous Wrath",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::count(Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature }),
            },
            Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
        ]),
        ..Default::default()
    }
}

/// Fire Nation's Conquest — {2}{R} enchantment: creatures you control get
/// +1/+0. (Zuko, Avatar Hunter.)
pub fn fire_nations_conquest() -> CardDefinition {
    CardDefinition {
        name: "Fire Nation's Conquest",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT { applies_to: yours(R::Creature), power: 1, toughness: 0 },
        }],
        ..Default::default()
    }
}

/// Zuko's Offense — {R} sorcery: 2 damage to any target. (Zuko, Avatar
/// Hunter.)
pub fn zukos_offense() -> CardDefinition {
    CardDefinition {
        name: "Zuko's Offense",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Lake-town — land; enters tapped; {T}: {W} or {U}. {2}{W}{U}, {T},
/// sacrifice it: two +1/+1 counters on target Human you control (sorcery).
/// (Sokka, Lateral Strategist.)
pub fn lake_town() -> CardDefinition {
    let tap = |c: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![c]) },
        ..Default::default()
    };
    CardDefinition {
        name: "Lake-town",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            tap(Color::White),
            tap(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w(), u()]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: plus_one(
                    target_filtered(R::HasCreatureType(CreatureType::Human).and(R::ControlledByYou)),
                    Value::Const(2),
                ),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Sokka, Wolf Cove's Protector — {2}{W} legendary 3/3 Human Warrior Ally;
/// vigilance. (Sokka, Lateral Strategist; Aang, Air Nomad.)
pub fn sokka_wolf_coves_protector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..legend(
            "Sokka, Wolf Cove's Protector",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            3,
            3,
        )
    }
}

/// Go-Shintai of Lost Wisdom — {1}{U} legendary 0/4 Shrine enchantment
/// creature; flying. Your end step: you may pay {1}; when you do, target
/// player mills X, X the Shrines you control.
pub fn go_shintai_of_lost_wisdom() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to have a player mill a card per Shrine?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::Mill { who: target_filtered(R::Player), amount: shrines() }),
                }),
                else_: None,
            },
        }],
        ..legend("Go-Shintai of Lost Wisdom", cost(&[generic(1), u()]), vec![], 0, 4)
    }
}

/// Sanctum of Calm Waters — {3}{U} legendary Shrine enchantment. Your first
/// main phase: you may draw X (the Shrines you control); if you do, discard
/// a card. (Go-Shintai of Lost Wisdom.)
pub fn sanctum_of_calm_waters() -> CardDefinition {
    CardDefinition {
        name: "Sanctum of Calm Waters",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Shrine], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Draw a card per Shrine, then discard a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: shrines() },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// King's Assassin — {1}{B}{B} 1/1 Human Assassin. {T}: destroy target
/// tapped creature; only during your turn, before attackers. (Jacob Frye.)
pub fn kings_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Any(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::CurrentStepIs(TurnStep::Draw),
                    Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                    Predicate::CurrentStepIs(TurnStep::BeginCombat),
                ]),
            ])),
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) },
            ..Default::default()
        }],
        ..creature("King's Assassin", cost(&[generic(1), b(), b()]), vec![CreatureType::Human, CreatureType::Assassin], 1, 1)
    }
}

/// Tainted Adversary — {1}{B} 2/3 Zombie; deathtouch. ETB: pay {2}{B} any
/// number of times; per payment a +1/+1 counter on it and two 2/2 decayed
/// Zombies. (Jacob Frye.)
pub fn tainted_adversary() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Decayed],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::MayPayRepeatedly {
            who: PlayerRef::You,
            description: "Pay {2}{B} for a counter and two decayed Zombies?".into(),
            mana_cost: cost(&[generic(2), b()]),
            body: Box::new(Effect::Seq(vec![
                plus_one(Selector::This, Value::ONE),
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(zombie) },
            ])),
        })],
        ..creature("Tainted Adversary", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 2, 3)
    }
}

/// Mockingbird, Bobbi Morse — {3}{W} legendary 3/3 Human Hero; whenever she
/// is dealt damage, a +1/+1 counter on her.
pub fn mockingbird_bobbi_morse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: plus_one(Selector::This, Value::ONE),
        }],
        ..legend("Mockingbird, Bobbi Morse", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3)
    }
}

/// D'Avenant Healer — {1}{W}{W} 1/2 Human Cleric Archer. {T}: 1 damage to
/// target attacking or blocking creature. {T}: prevent the next 1 damage to
/// any target this turn. (Mockingbird.)
pub fn davenant_healer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PreventNextDamage { target: target_any(), amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..creature(
            "D'Avenant Healer",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Archer],
            1,
            2,
        )
    }
}

/// Gandalf, White Rider — {3}{W} legendary 3/3 Avatar Wizard; vigilance.
/// Whenever you cast a spell, creatures you control get +1/+0 until end of
/// turn; scry 1. When it dies, you may put it into its owner's library fifth
/// from the top.
pub fn gandalf_white_rider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: yours(R::Creature),
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                ]),
            },
            on_dies(Effect::MayDo {
                description: "Put Gandalf into its owner's library fifth from the top?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                        pos: LibraryPosition::FromTop(4),
                    },
                }),
            }),
        ],
        ..legend("Gandalf, White Rider", cost(&[generic(3), w()]), vec![CreatureType::Avatar, CreatureType::Wizard], 3, 3)
    }
}

/// Niveous Wisps — {W} instant: target creature becomes white until end of
/// turn and taps; draw a card. (Gandalf, White Rider.)
pub fn niveous_wisps() -> CardDefinition {
    CardDefinition {
        name: "Niveous Wisps",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::BecomeColor {
                what: target_filtered(R::Creature),
                colors: vec![Color::White],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::Tap { what: Selector::Target(0) },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Collector's Cage — {1}{W} artifact; hideaway 5. {1}, {T}: a +1/+1
/// counter on target creature you control; then with three or more creatures
/// of different powers you may play the hidden card free. (Lord Jyscal Guado.)
pub fn collectors_cage() -> CardDefinition {
    CardDefinition {
        name: "Collector's Cage",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Hideaway { count: Value::Const(5) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                plus_one(target_filtered(R::Creature.and(R::ControlledByYou)), Value::ONE),
                Effect::If {
                    cond: Predicate::CovenActive { who: PlayerRef::You },
                    then: Box::new(Effect::MayDo {
                        description: "Play the hidden card without paying its mana cost?".into(),
                        body: Box::new(choose_one_then(
                            Selector::CardExiledWithSource,
                            PlayerRef::You,
                            Effect::If {
                                cond: Predicate::EntityMatches { what: chosen_one(), filter: R::Land },
                                then: Box::new(Effect::PlayLandAmongNow { what: chosen_one() }),
                                else_: Box::new(Effect::CastWithoutPayingImmediate {
                                    what: chosen_one(),
                                    source_zone: Zone::Exile,
                                    exile_after: false,
                                    copy: false,
                                    reduce_generic: 0,
                                    pay_own_cost: false,
                                }),
                            },
                        )),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gideon's Avenger — {1}{W}{W} 2/2 Human Soldier; whenever a creature an
/// opponent controls becomes tapped, a +1/+1 counter on it. (Lord Jyscal.)
pub fn gideons_avenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::OpponentControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature,
            }),
            effect: plus_one(Selector::This, Value::ONE),
        }],
        ..creature("Gideon's Avenger", cost(&[generic(1), w(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}
