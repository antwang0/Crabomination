//! Commander: 2026-10-19 (`01DJHkWk`) — the TMNT Foot cluster (Karai, Dark
//! Leo & Shredder, Shredder Unrelenting), Kate Stewart's vanishing deck, Chong
//! and Lily's Sagas, Bolg of the North's last card. Tests in
//! `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, w};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: chapters,
        ..spell(name, mana, CardType::Enchantment, Effect::Noop)
    }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn pump(what: Selector, power: Value, toughness: Value) -> Effect {
    Effect::PumpPT {
        what,
        power,
        toughness,
        duration: Duration::EndOfTurn,
    }
}

fn wb() -> crate::mana::ManaSymbol {
    hybrid(Color::White, Color::Black)
}

fn impulse(n: i32) -> Effect {
    Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::Const(n),
        duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    }
}

/// "You may discard a card. If you do, `then`."
fn may_discard_then(then: Effect) -> Effect {
    Effect::MayDo {
        description: "Discard a card?".into(),
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
                then: Box::new(then),
                else_: Box::new(Effect::Noop),
            },
        ])),
    }
}

fn sneak(mana: ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        marks_kicked: true,
        ..crate::effect::shortcut::sneak(mana)
    }
}

/// Gundabad Opportunist — {3}{R} 4/2 Goblin Rogue. Enters: exile your top
/// card; you may play it until the end of your next turn.
pub fn gundabad_opportunist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(impulse(1))],
        ..creature(
            "Gundabad Opportunist",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            4,
            2,
        )
    }
}

// ── The Foot ────────────────────────────────────────────────────────────────

/// Tunnel Rats — {1}{B} 2/2 Rat. {4}{B}: return it from your graveyard to the
/// battlefield tapped.
pub fn tunnel_rats() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Tunnel Rats",
            cost(&[generic(1), b()]),
            vec![CreatureType::Rat],
            2,
            2,
        )
    }
}

/// Shredder's Revenge — {2}{B} sorcery. Choose one: target player discards
/// two; or target player draws two and loses 2 life.
pub fn shredders_revenge() -> CardDefinition {
    spell(
        "Shredder's Revenge",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Discard {
                who: target_filtered(R::Player),
                amount: Value::Const(2),
                random: false,
            },
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
        ]),
    )
}

/// Foot Elite — {2}{W/B} 2/4 Human Ninja. Attacking, another target creature
/// you control gets +1/+0 and indestructible until end of turn.
pub fn foot_elite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            pump(
                target_filtered(mine().and(R::OtherThanSource)),
                Value::ONE,
                Value::Const(0),
            ),
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Foot Elite",
            cost(&[generic(2), wb()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            2,
            4,
        )
    }
}

/// Foot Ninjas — {4}{W/B}{W/B} 5/5 Human Ninja, sneak {3}{W/B}; gain 3 life as
/// it enters.
pub fn foot_ninjas() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(sneak(cost(&[generic(3), wb()]))),
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(3),
        })],
        ..creature(
            "Foot Ninjas",
            cost(&[generic(4), wb(), wb()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            5,
            5,
        )
    }
}

/// Uneasy Alliance — {1}{W} Aura: the creature can't attack or block. {5},
/// sacrifice it: exile the creature and create a 1/1 black Ninja; sorcery
/// speed.
pub fn uneasy_alliance() -> CardDefinition {
    let ninja = TokenDefinition {
        name: "Ninja".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ninja],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    to: ZoneDest::Exile,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(ninja),
                },
            ]),
            ..Default::default()
        }],
        ..spell(
            "Uneasy Alliance",
            cost(&[generic(1), w()]),
            CardType::Enchantment,
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
        )
    }
}

/// Karai's Technique — {1}{W}{B} sorcery, sneak {W}{B}. Choose one or both:
/// target creature gets +3/+3; target creature gets -3/-3; until end of turn.
pub fn karais_technique() -> CardDefinition {
    let three = |n: i32| {
        pump(
            target_filtered(R::Creature),
            Value::Const(n),
            Value::Const(n),
        )
    };
    CardDefinition {
        alternative_cost: Some(sneak(cost(&[w(), b()]))),
        ..spell(
            "Karai's Technique",
            cost(&[generic(1), w(), b()]),
            CardType::Sorcery,
            Effect::ChooseModesCast {
                modes: vec![three(3), three(-3)],
                min: 1,
                max: 2,
                allow_repeats: false,
            },
        )
    }
}

// ── Kate Stewart ────────────────────────────────────────────────────────────

/// Time Bomb — {4} artifact. Your upkeep adds a time counter. {1}, {T},
/// sacrifice it: damage equal to its time counters to each creature and each
/// player.
pub fn time_bomb() -> CardDefinition {
    let n = || Value::CountersOn {
        what: Box::new(Selector::This),
        kind: CounterType::Time,
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Time,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: n(),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: n(),
                },
            ]),
            ..Default::default()
        }],
        ..spell(
            "Time Bomb",
            cost(&[generic(4)]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Lavacore Elemental — {2}{R} 5/3 Elemental, vanishing 1. A creature of yours
/// dealing combat damage to a player puts a time counter on it.
pub fn lavacore_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vanishing(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Time,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Lavacore Elemental",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental],
            5,
            3,
        )
    }
}

/// Soultether Golem — {2} 3/3 Golem artifact creature, vanishing 1. Another
/// creature of yours entering puts a time counter on it.
pub fn soultether_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Vanishing(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Time,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Soultether Golem",
            cost(&[generic(2)]),
            vec![CreatureType::Golem],
            3,
            3,
        )
    }
}

/// Tidewalker — {2}{U} */* Elemental, vanishing; enters with a time counter per
/// Island you control, its P/T are the time counters on it.
pub fn tidewalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vanishing(0)],
        enters_with_counters: Some((
            CounterType::Time,
            Value::count(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Island),
            }),
        )),
        dynamic_pt: Some(DynamicPt::BasePlusCountersOnSelf {
            counter_type: CounterType::Time,
            base_p: 0,
            base_t: 0,
            per_p: 1,
            per_t: 1,
        }),
        ..creature(
            "Tidewalker",
            cost(&[generic(2), crate::mana::u()]),
            vec![CreatureType::Elemental],
            0,
            0,
        )
    }
}

// ── Chong and Lily, Nomads ──────────────────────────────────────────────────

/// Chong and Lily, Nomads — {3}{R} legendary 3/3 Human Bard Ally. Whenever one
/// or more Bards you control attack, choose one: a lore counter on each Saga
/// you control (the "any number of target Sagas" is every one); or your
/// creatures get +1/+0 for each lore counter among your Sagas.
pub fn chong_and_lily_nomads() -> CardDefinition {
    let sagas = || Selector::ControlledBy {
        who: PlayerRef::You,
        filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Saga),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Bard),
                })
                .once_per_batch(),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: sagas(),
                    kind: CounterType::Lore,
                    amount: Value::ONE,
                },
                pump(
                    Selector::EachPermanent(mine()),
                    Value::CountersOn {
                        what: Box::new(sagas()),
                        kind: CounterType::Lore,
                    },
                    Value::Const(0),
                ),
            ]),
        }],
        ..creature(
            "Chong and Lily, Nomads",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Bard, CreatureType::Ally],
            3,
            3,
        )
    }
}

/// Valor Singer — {2}{R} 2/3 Tiefling Bard. Combat Inspiration — at the
/// beginning of combat on your turn, target creature you control gets +1/+0.
pub fn valor_singer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: pump(target_filtered(mine()), Value::ONE, Value::Const(0)),
        }],
        ..creature(
            "Valor Singer",
            cost(&[generic(2), r()]),
            vec![CreatureType::Tiefling, CreatureType::Bard],
            2,
            3,
        )
    }
}

/// Founding of Omashu — {2}{R} Saga. I: two 1/1 white Allies. II: you may
/// discard a card; if you do, draw. III: your creatures get +1/+0.
pub fn founding_of_omashu() -> CardDefinition {
    let ally = TokenDefinition {
        name: "Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    };
    saga(
        "Founding of Omashu",
        cost(&[generic(2), r()]),
        vec![
            (
                1,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: Arc::new(ally),
                },
            ),
            (
                2,
                may_discard_then(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            ),
            (
                3,
                pump(Selector::EachPermanent(mine()), Value::ONE, Value::Const(0)),
            ),
        ],
    )
}

/// Ral and the Implicit Maze — {3}{R}{R} Saga. I: 2 damage to each creature
/// and planeswalker your opponents control. II: you may discard; if you do,
/// exile your top two, playable until the end of your next turn. III: a
/// Spellgorger Weird token.
pub fn ral_and_the_implicit_maze() -> CardDefinition {
    let weird = TokenDefinition {
        name: "Spellgorger Weird".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Weird],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature.negate())),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    };
    saga(
        "Ral and the Implicit Maze",
        cost(&[generic(3), r(), r()]),
        vec![
            (
                1,
                Effect::DealDamage {
                    to: Selector::EachPermanent(
                        R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
                    ),
                    amount: Value::Const(2),
                },
            ),
            (2, may_discard_then(impulse(2))),
            (
                3,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(weird),
                },
            ),
        ],
    )
}
