//! Ability-word conditional cards built on the new condition predicates:
//! Threshold, Metalcraft, Ferocious, Hellbent, and Formidable
//! (`Predicate::{ThresholdActive, MetalcraftActive, FerociousActive,
//! HellbentActive, FormidableActive}`). Tests in `tests/abilitywords.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Predicate, StaticAbility,
    StaticEffect, Subtypes,
};
use crate::effect::shortcut::on_attack;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, Value};
use crate::mana::{b, cost, g, generic, r, w, Color};

// ── Threshold (7+ cards in graveyard) ────────────────────────────────────────

/// Springing Tiger — {3}{G} 3/3 Cat. Threshold — gets +2/+2.
pub fn springing_tiger() -> CardDefinition {
    CardDefinition {
        name: "Springing Tiger",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 3,
        toughness: 3,
        static_abilities: vec![threshold_pump(2, 2, vec![])],
        ..Default::default()
    }
}

/// Mystic Enforcer — {2}{G}{W} 3/3 Human Nomad Mystic. Protection from black.
/// Threshold — gets +3/+3 and has flying.
pub fn mystic_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Mystic Enforcer",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Protection(Color::Black)],
        static_abilities: vec![threshold_pump(3, 3, vec![Keyword::Flying])],
        ..Default::default()
    }
}

/// Anurid Barkripper — {1}{G}{G} 2/2 Frog Beast. Threshold — gets +2/+2.
pub fn anurid_barkripper() -> CardDefinition {
    CardDefinition {
        name: "Anurid Barkripper",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![threshold_pump(2, 2, vec![])],
        ..Default::default()
    }
}

/// Krosan Beast — {3}{G} 1/1 Squirrel Beast. Threshold — gets +7/+7.
pub fn krosan_beast() -> CardDefinition {
    CardDefinition {
        name: "Krosan Beast",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![threshold_pump(7, 7, vec![])],
        ..Default::default()
    }
}

fn threshold_pump(power: i32, toughness: i32, keywords: Vec<Keyword>) -> StaticAbility {
    StaticAbility {
        description: "Threshold — gets +P/+T while seven or more cards are in your graveyard.",
        effect: StaticEffect::PumpSelfIf {
            condition: Predicate::ThresholdActive { who: PlayerRef::You },
            power,
            toughness,
            keywords,
        },
    }
}

// ── Metalcraft (control 3+ artifacts) ────────────────────────────────────────

/// Ardent Recruit — {W} 1/1 Human Soldier. Metalcraft — gets +2/+2.
pub fn ardent_recruit() -> CardDefinition {
    CardDefinition {
        name: "Ardent Recruit",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![metalcraft_pump(2, 2, vec![])],
        ..Default::default()
    }
}

/// Auriok Sunchaser — {1}{W} 1/1 Human Soldier. Metalcraft — gets +2/+2 and
/// has flying.
pub fn auriok_sunchaser() -> CardDefinition {
    CardDefinition {
        name: "Auriok Sunchaser",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![metalcraft_pump(2, 2, vec![Keyword::Flying])],
        ..Default::default()
    }
}

/// Snapsail Glider — {3} 2/2 Artifact Creature — Construct. Metalcraft — has
/// flying.
pub fn snapsail_glider() -> CardDefinition {
    CardDefinition {
        name: "Snapsail Glider",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 2,
        toughness: 2,
        static_abilities: vec![metalcraft_pump(0, 0, vec![Keyword::Flying])],
        ..Default::default()
    }
}

/// Dispatch — {W} Instant. Tap target creature. Metalcraft — exile it instead
/// if you control three or more artifacts.
pub fn dispatch() -> CardDefinition {
    CardDefinition {
        name: "Dispatch",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::MetalcraftActive { who: PlayerRef::You },
            then: Box::new(Effect::Exile { what: Selector::Target(0) }),
            else_: Box::new(Effect::Tap { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

fn metalcraft_pump(power: i32, toughness: i32, keywords: Vec<Keyword>) -> StaticAbility {
    StaticAbility {
        description: "Metalcraft — gets +P/+T while you control three or more artifacts.",
        effect: StaticEffect::PumpSelfIf {
            condition: Predicate::MetalcraftActive { who: PlayerRef::You },
            power,
            toughness,
            keywords,
        },
    }
}

// ── Ferocious (control a creature with power 4+) ─────────────────────────────

/// Savage Punch — {1}{G} Sorcery. Target creature you control fights target
/// creature you don't control. Ferocious — the creature you control gets +2/+2
/// first if you control a creature with power 4 or greater.
pub fn savage_punch() -> CardDefinition {
    CardDefinition {
        name: "Savage Punch",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::FerociousActive { who: PlayerRef::You },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight {
                attacker: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

// ── Formidable (creatures you control total power 8+) ─────────────────────────

/// Sabertooth Outrider — {3}{R} 4/2 Human Warrior. Trample. Formidable —
/// whenever this attacks, if creatures you control have total power 8 or
/// greater, it gains first strike until end of turn.
pub fn sabertooth_outrider() -> CardDefinition {
    CardDefinition {
        name: "Sabertooth Outrider",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::FormidableActive { who: PlayerRef::You },
            then: Box::new(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Circle of Elders — {2}{G}{G} 2/4 Human Shaman. Vigilance. Formidable —
/// {T}: Add {C}{C}{C}. Activate only if creatures you control have total power
/// 8 or greater.
pub fn circle_of_elders() -> CardDefinition {
    CardDefinition {
        name: "Circle of Elders",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::FormidableActive { who: PlayerRef::You }),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(3)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Hellbent (no cards in hand) ──────────────────────────────────────────────

/// Rakdos Pit Dragon — {2}{R}{R} 3/3 Dragon. {R}{R}: gains flying until end of
/// turn. {R}: gets +1/+0 until end of turn. Hellbent — has double strike while
/// you have no cards in hand.
pub fn rakdos_pit_dragon() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Pit Dragon",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r(), r()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Hellbent — has double strike while you have no cards in hand.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::HellbentActive { who: PlayerRef::You },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::DoubleStrike],
            },
        }],
        ..Default::default()
    }
}

/// Cutthroat il-Dal — {3}{B} 4/1 Human Rogue. Hellbent — has shadow while you
/// have no cards in hand.
pub fn cutthroat_il_dal() -> CardDefinition {
    CardDefinition {
        name: "Cutthroat il-Dal",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Hellbent — has shadow while you have no cards in hand.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::HellbentActive { who: PlayerRef::You },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Shadow],
            },
        }],
        ..Default::default()
    }
}

// ── Conditional-effect burn (non-ability-word) ───────────────────────────────

/// Bring Low — {3}{R} Instant. Deals 3 damage to target creature; 5 instead if
/// it has a +1/+1 counter on it.
pub fn bring_low() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Bring Low",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne)),
            },
            then: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) }),
            else_: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) }),
        },
        ..Default::default()
    }
}

/// Sarkhan's Rage — {4}{R} Instant. Deals 5 damage to any target. If you
/// control no Dragons, it deals 2 damage to you.
pub fn sarkhans_rage() -> CardDefinition {
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Sarkhan's Rage",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(5) },
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::ControlledByYou.and(R::HasCreatureType(CreatureType::Dragon)),
                    ),
                    n: Value::Const(1),
                })),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::You),
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}
