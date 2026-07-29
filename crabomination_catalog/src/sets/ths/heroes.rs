//! Theros (THS) — the heroic / bestow / Ordeal commons and uncommons.
//! Tests in `classic_sets/ths`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, heroic, target_any, target_filtered};
use crate::effect::{Duration, Effect, Predicate, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

fn creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: ct, ..Default::default() },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

/// An Aura that attaches to a creature and grants `bonus`.
fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// An enchantment creature with bestow: as an Aura it grants `+p/+t` and `kw`.
fn bestow_creature(
    name: &'static str,
    mana: ManaCost,
    bestow_cost: ManaCost,
    pt: (i32, i32),
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
    bonus_pt: (i32, i32),
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(bestow_cost),
        equipped_bonus: Some(EquipBonus {
            power: bonus_pt.0,
            toughness: bonus_pt.1,
            keywords: kw.clone(),
            ..Default::default()
        }),
        ..creature(name, mana, pt.0, pt.1, ct, kw)
    }
}

// ── Heroic ──────────────────────────────────────────────────────────────────

/// Akroan Crusader — {R} 1/1 Human Soldier. Heroic: create a 1/1 red Soldier
/// with haste.
pub fn akroan_crusader() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Soldier".into(),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Haste],
                card_types: vec![CardType::Creature],
                colors: vec![Color::Red],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Soldier],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..creature("Akroan Crusader", cost(&[r()]), 1, 1, vec![CreatureType::Human, CreatureType::Soldier], vec![])
    }
}

/// Battlewise Hoplite — {W}{U} 2/2 Human Soldier. Heroic: a +1/+1 counter,
/// then scry 1.
pub fn battlewise_hoplite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]))],
        ..creature("Battlewise Hoplite", cost(&[w(), u()]), 2, 2, vec![CreatureType::Human, CreatureType::Soldier], vec![])
    }
}

/// Favored Hoplite — {W} 1/2 Human Soldier. Heroic: a +1/+1 counter and
/// prevent all damage that would be dealt to it this turn.
pub fn favored_hoplite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::PreventAllDamageThisTurn { target: Selector::This },
        ]))],
        ..creature("Favored Hoplite", cost(&[w()]), 1, 2, vec![CreatureType::Human, CreatureType::Soldier], vec![])
    }
}

// ── Heroic-enabling instants ────────────────────────────────────────────────

/// Battlewise Valor — {1}{W} Instant. Target creature gets +2/+2; scry 1.
pub fn battlewise_valor() -> CardDefinition {
    CardDefinition {
        name: "Battlewise Valor",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Coordinated Assault — {R} Instant. Up to two target creatures each get
/// +1/+0 and gain first strike.
pub fn coordinated_assault() -> CardDefinition {
    CardDefinition {
        name: "Coordinated Assault",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
        ..Default::default()
    }
}

/// Dauntless Onslaught — {2}{W} Instant. Up to two target creatures each get
/// +2/+2 until end of turn.
pub fn dauntless_onslaught() -> CardDefinition {
    CardDefinition {
        name: "Dauntless Onslaught",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Chosen by Heliod — {1}{W} Aura. ETB draw a card; enchanted creature gets
/// +0/+2.
pub fn chosen_by_heliod() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..aura(
            "Chosen by Heliod",
            cost(&[generic(1), w()]),
            EquipBonus { toughness: 2, ..Default::default() },
        )
    }
}

/// Fate Foretold — {1}{U} Aura. ETB draw a card; when enchanted creature dies,
/// its controller draws a card.
pub fn fate_foretold() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                        Selector::TriggerSource,
                    ))),
                    amount: Value::ONE,
                },
            },
        ],
        ..aura("Fate Foretold", cost(&[generic(1), u()]), EquipBonus::default())
    }
}

/// Feral Invocation — {2}{G} Aura with flash. Enchanted creature gets +2/+2.
pub fn feral_invocation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..aura(
            "Feral Invocation",
            cost(&[generic(2), g()]),
            EquipBonus { power: 2, toughness: 2, ..Default::default() },
        )
    }
}

/// Dragon Mantle — {R} Aura. ETB draw a card; enchanted creature has
/// "{R}: this creature gets +1/+0 until end of turn."
pub fn dragon_mantle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..aura(
            "Dragon Mantle",
            cost(&[r()]),
            EquipBonus {
                activated_abilities: vec![ActivatedAbility {
                    mana_cost: cost(&[r()]),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Messenger's Speed — {R} Aura. Enchanted creature has trample and haste.
pub fn messengers_speed() -> CardDefinition {
    aura(
        "Messenger's Speed",
        cost(&[r()]),
        EquipBonus { keywords: vec![Keyword::Trample, Keyword::Haste], ..Default::default() },
    )
}

// ── Bestow ──────────────────────────────────────────────────────────────────

/// Leafcrown Dryad — {1}{G} 2/2 Nymph Dryad with reach. Bestow {3}{G}: +2/+2
/// and reach.
pub fn leafcrown_dryad() -> CardDefinition {
    bestow_creature(
        "Leafcrown Dryad",
        cost(&[generic(1), g()]),
        cost(&[generic(3), g()]),
        (2, 2),
        vec![CreatureType::Nymph, CreatureType::Dryad],
        vec![Keyword::Reach],
        (2, 2),
    )
}

/// Nimbus Naiad — {2}{U} 2/2 Nymph with flying. Bestow {4}{U}: +2/+2 and
/// flying.
pub fn nimbus_naiad() -> CardDefinition {
    bestow_creature(
        "Nimbus Naiad",
        cost(&[generic(2), u()]),
        cost(&[generic(4), u()]),
        (2, 2),
        vec![CreatureType::Nymph],
        vec![Keyword::Flying],
        (2, 2),
    )
}

/// Observant Alseid — {2}{W} 2/2 Nymph with vigilance. Bestow {4}{W}: +2/+2
/// and vigilance.
pub fn observant_alseid() -> CardDefinition {
    bestow_creature(
        "Observant Alseid",
        cost(&[generic(2), w()]),
        cost(&[generic(4), w()]),
        (2, 2),
        vec![CreatureType::Nymph],
        vec![Keyword::Vigilance],
        (2, 2),
    )
}

/// Cavern Lampad — {3}{B} 2/2 Nymph with intimidate. Bestow {5}{B}: +2/+2 and
/// intimidate.
pub fn cavern_lampad() -> CardDefinition {
    bestow_creature(
        "Cavern Lampad",
        cost(&[generic(3), b()]),
        cost(&[generic(5), b()]),
        (2, 2),
        vec![CreatureType::Nymph],
        vec![Keyword::Intimidate],
        (2, 2),
    )
}

/// Nylea's Emissary — {3}{G} 3/3 Cat with trample. Bestow {5}{G}: +3/+3 and
/// trample.
pub fn nyleas_emissary() -> CardDefinition {
    bestow_creature(
        "Nylea's Emissary",
        cost(&[generic(3), g()]),
        cost(&[generic(5), g()]),
        (3, 3),
        vec![CreatureType::Cat],
        vec![Keyword::Trample],
        (3, 3),
    )
}

/// Heliod's Emissary — {3}{W} 3/3 Elk. Bestow {6}{W}: +3/+3. Whenever it or
/// the enchanted creature attacks, tap target creature an opponent controls.
pub fn heliods_emissary() -> CardDefinition {
    let tap_on_attack = TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::Tap {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        },
    };
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(cost(&[generic(6), w()])),
        triggered_abilities: vec![tap_on_attack.clone()],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            triggered_abilities: vec![tap_on_attack],
            ..Default::default()
        }),
        ..creature("Heliod's Emissary", cost(&[generic(3), w()]), 3, 3, vec![CreatureType::Elk], vec![])
    }
}

// ── Ordeals ─────────────────────────────────────────────────────────────────

/// The Ordeal cycle: whenever the enchanted creature attacks, put a +1/+1
/// counter on it, then sacrifice the Aura (and run `payoff`) once it has three.
fn ordeal(name: &'static str, mana: ManaCost, payoff: Effect) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::TriggerSource),
                            kind: CounterType::PlusOnePlusOne,
                        },
                        Value::Const(3),
                    ),
                    // Payoff first: the printed "when you sacrifice this Aura"
                    // is a reflexive trigger that resolves after the sacrifice,
                    // but running it before keeps the Aura's bound target alive
                    // through its own removal.
                    then: Box::new(Effect::Seq(vec![payoff, Effect::SacrificeSource])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..aura(name, mana, EquipBonus::default())
    }
}

/// Ordeal of Heliod — {1}{W}. Sacrificed: you gain 10 life.
pub fn ordeal_of_heliod() -> CardDefinition {
    ordeal(
        "Ordeal of Heliod",
        cost(&[generic(1), w()]),
        Effect::GainLife { who: Selector::You, amount: Value::Const(10) },
    )
}

/// Ordeal of Thassa — {1}{U}. Sacrificed: draw two cards.
pub fn ordeal_of_thassa() -> CardDefinition {
    ordeal(
        "Ordeal of Thassa",
        cost(&[generic(1), u()]),
        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
    )
}

/// Ordeal of Erebos — {1}{B}. Sacrificed: target player discards two cards.
pub fn ordeal_of_erebos() -> CardDefinition {
    ordeal(
        "Ordeal of Erebos",
        cost(&[generic(1), b()]),
        Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
            random: false,
        },
    )
}

/// Ordeal of Purphoros — {1}{R}. Sacrificed: 3 damage to any target.
pub fn ordeal_of_purphoros() -> CardDefinition {
    ordeal(
        "Ordeal of Purphoros",
        cost(&[generic(1), r()]),
        Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
    )
}

/// Ordeal of Nylea — {1}{G}. Sacrificed: search for up to two basic lands,
/// tapped.
pub fn ordeal_of_nylea() -> CardDefinition {
    ordeal(
        "Ordeal of Nylea",
        cost(&[generic(1), g()]),
        Effect::Repeat {
            count: Value::Const(2),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        },
    )
}
