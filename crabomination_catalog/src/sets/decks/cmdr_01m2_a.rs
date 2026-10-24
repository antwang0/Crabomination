//! Commander: 2026-10-24 (`01M2Ffkh`) — Isu the Abominable's snow deck and
//! Scarlet Witch, Wanda Maximoff's. Tests in `tests/recent_b/cmdr_01m2_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{ManaCost, cost, g, generic, r, snow_mana, u, w};

fn snow_creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
    }
}

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn cumulative_snow() -> Keyword {
    Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[snow_mana()])))
}

/// Isu the Abominable — {3}{U}{U} legendary snow 5/5 Yeti. Look at the top
/// card of your library any time; play snow lands and cast snow spells from
/// it. Another snow permanent of yours entering lets you pay {G}, {W} or {U}
/// for a +1/+1 counter (the colour is a mode, then the may-pay).
pub fn isu_the_abominable() -> CardDefinition {
    let counter = || Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    let pay = |pip| Effect::MayPay {
        description: "Pay for a +1/+1 counter on Isu?".into(),
        mana_cost: cost(&[pip]),
        body: Box::new(counter()),
        else_: None,
    };
    let trigger = Effect::ChooseMode(vec![pay(g()), pay(w()), pay(u())]);
    CardDefinition {
        supertypes: vec![Supertype::Legendary, Supertype::Snow],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may play snow lands and cast snow spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::IsSnow },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSnow }),
            effect: trigger,
        }],
        ..snow_creature("Isu the Abominable", cost(&[generic(3), u(), u()]), vec![CreatureType::Yeti], 5, 5)
    }
}

/// Diamond Faerie — {2}{G}{W}{U} snow 3/3 flying Faerie. {1}{S}: snow
/// creatures you control get +1/+1 until end of turn.
pub fn diamond_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), snow_mana()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsSnow).and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..snow_creature("Diamond Faerie", cost(&[generic(2), g(), w(), u()]), vec![CreatureType::Faerie], 3, 3)
    }
}

/// Frostpeak Yeti — {3}{U} snow 3/3 Yeti. {1}{S}: it can't be blocked this turn.
pub fn frostpeak_yeti() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), snow_mana()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..snow_creature("Frostpeak Yeti", cost(&[generic(3), u()]), vec![CreatureType::Yeti], 3, 3)
    }
}

/// Squall Drifter — {1}{W} snow 1/1 flying Elemental. {W}, {T}: tap target creature.
pub fn squall_drifter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w()]),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..snow_creature("Squall Drifter", cost(&[generic(1), w()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Snow Devil — {1}{U} Aura. Enchanted creature has flying, and first strike
/// as long as it's blocking and you control a snow land.
pub fn snow_devil() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has first strike as long as it's blocking and you control a snow land.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::All(vec![
                    Predicate::EntityMatches { what: host(), filter: R::IsBlocking },
                    Predicate::SelectorExists(Selector::EachPermanent(
                        R::Land.and(R::IsSnow).and(R::ControlledByYou),
                    )),
                ]),
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::FirstStrike }),
            },
        }],
        ..aura("Snow Devil", cost(&[generic(1), u()]))
    }
}

/// Glacial Plating — {2}{W}{W} snow Aura. Cumulative upkeep {S}; enchanted
/// creature gets +3/+3 for each age counter on it.
pub fn glacial_plating() -> CardDefinition {
    let x = || Value::Times(
        Box::new(Value::Const(3)),
        Box::new(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Age }),
    );
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        keywords: vec![cumulative_snow()],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +3/+3 for each age counter on this Aura.",
            effect: StaticEffect::PumpPTByValue { applies_to: host(), power: x(), toughness: x() },
        }],
        ..aura("Glacial Plating", cost(&[generic(2), w(), w()]))
    }
}

/// Cover of Winter — {2}{W} snow enchantment. Cumulative upkeep {S}; X of
/// each creature's combat damage to you or your creatures is prevented, X its
/// age counters; {S}: another age counter.
pub fn cover_of_winter() -> CardDefinition {
    CardDefinition {
        name: "Cover of Winter",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Enchantment],
        keywords: vec![cumulative_snow()],
        static_abilities: vec![StaticAbility {
            description: "If a creature would deal combat damage to you and/or one or more creatures you control, prevent X of that damage, where X is the number of age counters on this enchantment.",
            effect: StaticEffect::ReduceCreatureCombatDamageToYouAndYoursByCounters { counter: CounterType::Age },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[snow_mana()]),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Age, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Scarlet Witch, Wanda Maximoff's deck ────────────────────────────────────

fn red_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn red_spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// Barging Sergeant — {4}{R} 4/2 haste Minotaur Soldier with mentor.
pub fn barging_sergeant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::effect::shortcut::mentor()],
        ..red_creature("Barging Sergeant", cost(&[generic(4), r()]), vec![CreatureType::Minotaur, CreatureType::Soldier], 4, 2)
    }
}

/// Bladeback Sliver — {1}{R} 2/2 Sliver. Hellbent: your Slivers have "{T}:
/// this creature deals 1 damage to target player or planeswalker."
pub fn bladeback_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Hellbent — As long as you have no cards in hand, Sliver creatures you control have \"{T}: This creature deals 1 damage to target player or planeswalker.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Sliver)).and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::ONE },
                    ..Default::default()
                },
                condition: Some(Predicate::HellbentActive { who: PlayerRef::You }),
            },
        }],
        ..red_creature("Bladeback Sliver", cost(&[generic(1), r()]), vec![CreatureType::Sliver], 2, 2)
    }
}

/// Hellkite Whelp — {4}{R} 3/3 flying Dragon. Attacking, it deals 1 damage to
/// target creature defending player controls.
pub fn hellkite_whelp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                amount: Value::ONE,
            },
        }],
        ..red_creature("Hellkite Whelp", cost(&[generic(4), r()]), vec![CreatureType::Dragon], 3, 3)
    }
}

/// Voldaren Duelist — {3}{R} 3/2 haste Vampire Warrior. ETB: target creature
/// can't block this turn.
pub fn voldaren_duelist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..red_creature("Voldaren Duelist", cost(&[generic(3), r()]), vec![CreatureType::Vampire, CreatureType::Warrior], 3, 2)
    }
}

/// Prophetic Ravings — {R} Aura. Enchanted creature has haste and "{T},
/// Discard a card: Draw a card."
pub fn prophetic_ravings() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Haste],
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Prophetic Ravings", cost(&[r()]))
    }
}

/// Volatile Claws — {2}{R} instant. Your creatures get +2/+0 and gain all
/// creature types until end of turn.
pub fn volatile_claws() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    red_spell(
        "Volatile Claws",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT { what: yours(), power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: yours(), keyword: Keyword::Changeling, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Precision Bolt — {2}{R} sorcery. 3 damage to any target.
pub fn precision_bolt() -> CardDefinition {
    red_spell(
        "Precision Bolt",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) },
    )
}

/// Vandalize — {4}{R} sorcery. Choose one or both — destroy target artifact;
/// destroy target land.
pub fn vandalize() -> CardDefinition {
    red_spell(
        "Vandalize",
        cost(&[generic(4), r()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::Destroy { what: target_filtered(R::Land) },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}
