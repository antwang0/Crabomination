//! Commander: the cards the **Mirror Mastery** precon (CMD, Riku of Two
//! Reflections) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_riku.rs`.
//!
//! Residuals (each also on its card):
//! - **Intet, the Dreamer** — the card is exiled face up.
//! - **Ray of Command** — the creature is tapped at the next end step rather
//!   than as its control returns (the same turn; it stays tapped either way).

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, etb, evoke, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{cost, g, generic, hybrid, r, u, Color, ManaCost};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

/// Riku of Two Reflections — {2}{G}{U}{R} 2/2 legendary Human Wizard. Cast an
/// instant or sorcery → you may pay {U}{R} to copy it (new targets allowed);
/// another nontoken creature of yours enters → you may pay {G}{U} for a token
/// copy of it.
pub fn riku_of_two_reflections() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(cast_is_instant_or_sorcery()),
                effect: Effect::MayPay {
                    description: "Pay {U}{R} to copy that spell?".into(),
                    mana_cost: cost(&[u(), r()]),
                    body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
                    else_: None,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::NotToken) },
                ),
                effect: Effect::MayPay {
                    description: "Pay {G}{U} to create a token copy of it?".into(),
                    mana_cost: cost(&[g(), u()]),
                    body: Box::new(Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::TriggerSource,
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    }),
                    else_: None,
                },
            },
        ],
        ..legend(
            "Riku of Two Reflections",
            cost(&[generic(2), g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Aethersnipe — {5}{U} 4/4 Elemental. ETB: return target nonland permanent to
/// its owner's hand. Evoke {1}{U}{U}.
pub fn aethersnipe() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(evoke(cost(&[generic(1), u(), u()]))),
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Permanent.and(R::Nonland)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature("Aethersnipe", cost(&[generic(5), u()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Animar, Soul of Elements — {G}{U}{R} 1/1 legendary Elemental. Protection
/// from white and from black; grows on each creature spell you cast; your
/// creature spells cost {1} less per +1/+1 counter on it.
pub fn animar_soul_of_elements() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::White), Keyword::Protection(Color::Black)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCardType(CardType::Creature) },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast cost {1} less to cast for each +1/+1 counter on Animar.",
            effect: StaticEffect::CostReductionPerCounterOnSource {
                filter: R::HasCardType(CardType::Creature),
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        ..legend("Animar, Soul of Elements", cost(&[g(), u(), r()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Colossal Might — {R}{G} Instant. Target creature gets +4/+2 and trample
/// until end of turn.
pub fn colossal_might() -> CardDefinition {
    spell(
        "Colossal Might",
        cost(&[r(), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Deadwood Treefolk — {5}{G} 3/6 Treefolk. Vanishing 3; entering or leaving,
/// return another target creature card from your graveyard to your hand.
pub fn deadwood_treefolk() -> CardDefinition {
    let regrow = || Effect::Move {
        what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::OtherThanSource)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        keywords: vec![Keyword::Vanishing(3)],
        triggered_abilities: vec![
            etb(regrow()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: regrow(),
            },
        ],
        ..creature("Deadwood Treefolk", cost(&[generic(5), g()]), vec![CreatureType::Treefolk], 3, 6)
    }
}

/// Faultgrinder — {6}{R} 4/4 trample Elemental. ETB: destroy target land.
/// Evoke {4}{R}.
pub fn faultgrinder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        alternative_cost: Some(evoke(cost(&[generic(4), r()]))),
        triggered_abilities: vec![etb(Effect::Destroy { what: target_filtered(R::Land) })],
        ..creature("Faultgrinder", cost(&[generic(6), r()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Firespout — {2}{R/G} Sorcery. 3 damage to each creature without flying if
/// {R} was spent, to each creature with flying if {G} was spent.
pub fn firespout() -> CardDefinition {
    let flying = || R::HasKeyword(Keyword::Flying);
    let if_spent = |color, filter| Effect::If {
        cond: Predicate::ManaSpentOfColorAtLeast { color, at_least: 1 },
        then: Box::new(Effect::DealDamage { to: Selector::EachPermanent(filter), amount: Value::Const(3) }),
        else_: Box::new(Effect::Noop),
    };
    spell(
        "Firespout",
        cost(&[generic(2), hybrid(Color::Red, Color::Green)]),
        CardType::Sorcery,
        Effect::Seq(vec![
            if_spent(Color::Red, R::Creature.and(R::Not(Box::new(flying())))),
            if_spent(Color::Green, R::Creature.and(flying())),
        ]),
    )
}

/// Fungal Reaches — {T}: {C}; {1},{T}: a storage counter; {1}, remove X
/// storage counters: X mana in any combination of {R} and {G}.
pub fn fungal_reaches() -> CardDefinition {
    CardDefinition {
        name: "Fungal Reaches",
        card_types: vec![CardType::Land],
        activated_abilities: storage_land(Color::Red, Color::Green),
        ..Default::default()
    }
}

fn storage_land(a: Color, b: Color) -> Vec<crate::card::ActivatedAbility> {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    vec![
        crate::sets::tap_add_colorless(),
        ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Storage, amount: Value::ONE },
            ..Default::default()
        },
        ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_x: Some(CounterType::Storage),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![a, b], Value::XFromCost) },
            ..Default::default()
        },
    ]
}

/// Hydra Omnivore — {4}{G}{G} 8/8 Hydra. Combat damage to an opponent is also
/// dealt to each other opponent.
pub fn hydra_omnivore() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![combat_damage_to_player(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
            amount: Value::TriggerEventAmount,
        })],
        ..creature("Hydra Omnivore", cost(&[generic(4), g(), g()]), vec![CreatureType::Hydra], 8, 8)
    }
}

/// Intet, the Dreamer — {3}{G}{U}{R} 6/6 legendary flying Dragon. On combat
/// damage to a player you may pay {2}{U}: exile your top card; you may play it
/// free for as long as Intet remains on the battlefield. (Residual: face up.)
pub fn intet_the_dreamer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            combat_damage_to_player(Effect::MayPay {
                description: "Pay {2}{U} to exile your top card to play for free?".into(),
                mana_cost: cost(&[generic(2), u()]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                        to: ZoneDest::ExileWithSourceStamp,
                    },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: crate::card::MayPlayDuration::WhileExiled,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: false,
                        any_color: false,
                    },
                ])),
                else_: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::EndMayPlayOnCardsExiledWithSource,
            },
        ],
        ..legend(
            "Intet, the Dreamer",
            cost(&[generic(3), g(), u(), r()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Magus of the Vineyard — {G} 1/1 Human Wizard. Each player's first main
/// phase starts with {G}{G} in their pool.
pub fn magus_of_the_vineyard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::PreCombatMain),
                EventScope::AnyPlayer,
            ),
            effect: Effect::AddMana {
                who: PlayerRef::ActivePlayer,
                pool: crate::effect::ManaPayload::Colors(vec![Color::Green, Color::Green]),
            },
        }],
        ..creature("Magus of the Vineyard", cost(&[g()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 1)
    }
}

/// Nucklavee — {4}{U/R}{U/R} 4/4 Beast. ETB: you may return target red sorcery
/// card, and target blue instant card, from your graveyard to your hand.
pub fn nucklavee() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    let regrow = |kind: CardType, color: Color| {
        etb(Effect::MayDo {
            description: "Return it to your hand?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::HasCardType(kind).and(R::HasColor(color)).and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })
    };
    CardDefinition {
        triggered_abilities: vec![regrow(CardType::Sorcery, Color::Red), regrow(CardType::Instant, Color::Blue)],
        ..creature("Nucklavee", cost(&[generic(4), ur(), ur()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Ray of Command — {3}{U} Instant. Untap target creature an opponent controls
/// and gain control of it until end of turn; it gains haste. When you lose
/// control of it, tap it. (Residual: tapped at the next end step.)
pub fn ray_of_command() -> CardDefinition {
    spell(
        "Ray of Command",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::AtNextEndStep { body: Box::new(Effect::Tap { what: Selector::Target(0) }) },
        ]),
    )
}

/// Trench Gorger — {6}{U}{U} 6/6 trample Leviathan. ETB: you may exile any
/// number of land cards from your library; its base P/T become that number.
pub fn trench_gorger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile land cards from your library to set its base power and toughness?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::Land,
                    to: ZoneDest::ExileWithSourceStamp,
                    count: Value::Const(100),
                },
                Effect::SetBasePT {
                    what: Selector::This,
                    power: Value::CardsExiledWithSourceCount,
                    toughness: Value::CardsExiledWithSourceCount,
                    duration: Duration::Permanent,
                },
            ])),
        })],
        ..creature("Trench Gorger", cost(&[generic(6), u(), u()]), vec![CreatureType::Leviathan], 6, 6)
    }
}

/// Valley Rannet — {4}{R}{G} 6/3 Beast. Mountaincycling {2}, forestcycling {2}.
pub fn valley_rannet() -> CardDefinition {
    let landcycling = |t| Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::HasLandType(t))));
    CardDefinition {
        keywords: vec![landcycling(LandType::Mountain), landcycling(LandType::Forest)],
        ..creature("Valley Rannet", cost(&[generic(4), r(), g()]), vec![CreatureType::Beast], 6, 3)
    }
}

/// Vengeful Rebirth — {4}{R}{G} Sorcery. Return target card from your
/// graveyard to your hand; a nonland card deals damage equal to its mana value
/// to any target. Exile Vengeful Rebirth.
pub fn vengeful_rebirth() -> CardDefinition {
    spell(
        "Vengeful Rebirth",
        cost(&[generic(4), r(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::InYourGraveyard), to: ZoneDest::Hand(PlayerRef::You) },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Nonland },
                then: Box::new(Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.or(R::Player).or(R::Planeswalker),
                    },
                    amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::ExileResolvingSpell,
        ]),
    )
}
