//! Odyssey (ODY) gap-closing wave 3: the Cleric / Wizard / Bird shells and the
//! discard-fuelled creatures. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector,
    shortcut::{target_any, target_filtered},
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

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

/// "Discard a card: this creature gains `keyword` until end of turn."
fn pitch_for_keyword(keyword: Keyword, mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        discard_cost: Some((R::Any, 1)),
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// "Discard a card: return this creature to its owner's hand."
fn pitch_to_bounce(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        discard_cost: Some((R::Any, 1)),
        effect: Effect::Move {
            what: Selector::This,
            to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// "Threshold — this creature gets +P/+T and `keywords`."
fn threshold_pump(power: i32, toughness: i32, keywords: Vec<Keyword>) -> StaticAbility {
    StaticAbility {
        description: "Threshold — this creature gets a bonus.",
        effect: StaticEffect::PumpSelfIf { condition: threshold(), power, toughness, keywords },
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Hallowed Healer — {2}{W} 1/1 shield, doubled past Threshold.
pub fn hallowed_healer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PreventNextDamage {
                    target: target_any(),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(threshold()),
                effect: Effect::PreventNextDamage {
                    target: target_any(),
                    amount: Value::Const(4),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Hallowed Healer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Master Apothecary — {W}{W}{W} 2/2 that taps Clerics for shields.
pub fn master_apothecary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::HasCreatureType(CreatureType::Cleric))),
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature(
            "Master Apothecary",
            cost(&[w(), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Devoted Caretaker — {W} 1/2 that shrugs off removal spells.
pub fn devoted_caretaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                keyword: Keyword::ProtectionFromInstants,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Devoted Caretaker",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Resilient Wanderer — {2}{W}{W} 2/3 first striker that pitches for
/// protection.
pub fn resilient_wanderer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantProtectionFromChosenColor {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Resilient Wanderer",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            2,
            3,
        )
    }
}

/// Aven Archer — {3}{W}{W} 2/2 flier that snipes a combatant.
pub fn aven_archer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Aven Archer",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier, CreatureType::Archer],
            2,
            2,
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Aven Smokeweaver — {2}{U}{U} 2/3 flier with protection from red.
pub fn aven_smokeweaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        ..creature(
            "Aven Smokeweaver",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Treetop Sentinel — {2}{U}{U} 2/3 flier with protection from green.
pub fn treetop_sentinel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Green)],
        ..creature(
            "Treetop Sentinel",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Balshan Griffin — {3}{U}{U} 3/2 flier that ducks removal.
pub fn balshan_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![pitch_to_bounce(cost(&[generic(1), u()]))],
        ..creature(
            "Balshan Griffin",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Griffin],
            3,
            2,
        )
    }
}

/// Amugaba — {5}{U}{U} 6/6 flier that ducks removal.
pub fn amugaba() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![pitch_to_bounce(cost(&[generic(2), u()]))],
        ..creature(
            "Amugaba",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Illusion],
            6,
            6,
        )
    }
}

/// Thought Eater — {1}{U} 2/2 flier costing three cards of hand size.
pub fn thought_eater() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Your maximum hand size is reduced by three.",
            effect: StaticEffect::ControllerMaxHandSizeReduced(3),
        }],
        ..creature("Thought Eater", cost(&[generic(1), u()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Thought Devourer — {2}{U}{U} 4/4 flier costing four cards of hand size.
pub fn thought_devourer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Your maximum hand size is reduced by four.",
            effect: StaticEffect::ControllerMaxHandSizeReduced(4),
        }],
        ..creature(
            "Thought Devourer",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Beast],
            4,
            4,
        )
    }
}

/// Cephalid Retainer — {2}{U}{U} 2/3 that taps down the ground.
pub fn cephalid_retainer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            effect: Effect::Tap {
                what: target_filtered(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Cephalid Retainer",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Octopus],
            2,
            3,
        )
    }
}

/// Aboshan, Cephalid Emperor — {4}{U}{U} 3/3 that taps the board down.
pub fn aboshan_cephalid_emperor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_other_filter: Some(
                    R::Creature.and(R::HasCreatureType(CreatureType::Octopus)),
                ),
                effect: Effect::Tap { what: target_filtered(R::Permanent) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u(), u(), u()]),
                effect: Effect::Tap {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                    ),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Aboshan, Cephalid Emperor",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Octopus, CreatureType::Noble],
            3,
            3,
        )
    }
}

/// Patron Wizard — {U}{U}{U} 2/2 that taps Wizards to tax spells.
pub fn patron_wizard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::HasCreatureType(CreatureType::Wizard))),
            effect: Effect::CounterUnless {
                what: Selector::Target(0),
                cost: crate::card::WardCost::Mana(cost(&[generic(1)])),
            },
            ..Default::default()
        }],
        ..creature(
            "Patron Wizard",
            cost(&[u(), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Childhood Horror — {3}{B} 2/2 flier that swells past Threshold.
pub fn childhood_horror() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![threshold_pump(2, 2, vec![Keyword::CantBlock])],
        ..creature("Childhood Horror", cost(&[generic(3), b()]), vec![CreatureType::Horror], 2, 2)
    }
}

/// Bloodcurdler — {1}{B} 1/1 flier that self-mills and grows past Threshold.
pub fn bloodcurdler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Mill { who: Selector::You, amount: Value::ONE },
        }],
        static_abilities: vec![threshold_pump(1, 1, vec![])],
        ..creature("Bloodcurdler", cost(&[generic(1), b()]), vec![CreatureType::Horror], 1, 1)
    }
}

/// Fledgling Imp — {2}{B} 2/2 that pitches for flight.
pub fn fledgling_imp() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![pitch_for_keyword(Keyword::Flying, cost(&[b()]))],
        ..creature("Fledgling Imp", cost(&[generic(2), b()]), vec![CreatureType::Imp], 2, 2)
    }
}

/// Face of Fear — {5}{B} 3/4 that pitches for evasion.
pub fn face_of_fear() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![pitch_for_keyword(Keyword::Fear, cost(&[generic(2), b()]))],
        ..creature("Face of Fear", cost(&[generic(5), b()]), vec![CreatureType::Horror], 3, 4)
    }
}

/// Infected Vermin — {2}{B} 1/1 pinger that scales past Threshold.
pub fn infected_vermin() -> CardDefinition {
    let sweep = |n: i32| Effect::DealDamage {
        to: Selector::Both(
            Box::new(Selector::EachPermanent(R::Creature)),
            Box::new(Selector::Player(PlayerRef::EachPlayer)),
        ),
        amount: Value::Const(n),
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                effect: sweep(1),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b()]),
                condition: Some(threshold()),
                effect: sweep(3),
                ..Default::default()
            },
        ],
        ..creature("Infected Vermin", cost(&[generic(2), b()]), vec![CreatureType::Rat], 1, 1)
    }
}

/// Painbringer — {2}{B}{B} 1/1 that trades graveyard cards for -X/-X.
pub fn painbringer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_other_filter: Some((R::InYourGraveyard, 1)),
            exile_other_x: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Painbringer",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            1,
            1,
        )
    }
}

/// Zombie Assassin — {4}{B} 3/2 that eats a nonblack creature for good.
pub fn zombie_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            exile_other_filter: Some((R::InYourGraveyard, 2)),
            effect: Effect::DestroyNoRegen {
                what: target_filtered(
                    R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Zombie Assassin",
            cost(&[generic(4), b()]),
            vec![CreatureType::Zombie, CreatureType::Assassin],
            3,
            2,
        )
    }
}

// ── Green / red odds and ends ───────────────────────────────────────────────

/// Chlorophant — {G}{G}{G} 1/1 that grows each upkeep, twice past Threshold.
pub fn chlorophant() -> CardDefinition {
    let grow = |extra: bool| TriggeredAbility {
        event: {
            let spec = EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            );
            if extra { spec.with_filter(threshold()) } else { spec }
        },
        effect: Effect::MayDo {
            description: "Put a +1/+1 counter on Chlorophant?".into(),
            body: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        },
    };
    CardDefinition {
        triggered_abilities: vec![grow(false), grow(true)],
        ..creature("Chlorophant", cost(&[g(), g(), g()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Spark Mage — {R} 1/1 that pings on connection.
pub fn spark_mage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Ping a creature that player controls?".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature(
            "Spark Mage",
            cost(&[r()]),
            vec![CreatureType::Dwarf, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Whipkeeper — {2}{R}{R} 1/1 that doubles the damage already on a creature.
pub fn whipkeeper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::MarkedDamageOn(Box::new(Selector::Target(0))),
            },
            ..Default::default()
        }],
        ..creature("Whipkeeper", cost(&[generic(2), r(), r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}
