//! Tempest (TMP) artifacts. Tests in `classic_sets/tmp`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, StaticEffect, Value};
use crate::game::TurnStep;
use crate::mana::{ManaCost, cost, generic};

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn book(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        ..artifact(name, c, abilities)
    }
}

/// Emmessi Tome — {4}. {5}, {T}: Draw two cards, then discard a card.
pub fn emmessi_tome() -> CardDefinition {
    book(
        "Emmessi Tome",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                crate::effect::shortcut::draw(2),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
    )
}

/// Fool's Tome — {4}. {2}, {T}: Draw a card, but only on an empty hand.
pub fn fools_tome() -> CardDefinition {
    book(
        "Fool's Tome",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            condition: Some(Predicate::HellbentActive { who: PlayerRef::You }),
            effect: crate::effect::shortcut::draw(1),
            ..Default::default()
        }],
    )
}

/// Essence Bottle — {2}. Bank elixir counters, then cash them in for life.
pub fn essence_bottle() -> CardDefinition {
    artifact(
        "Essence Bottle",
        cost(&[generic(2)]),
        vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Elixir,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_all_counters_cost: Some(CounterType::Elixir),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::CountersRemovedAsCost),
                    ),
                },
                ..Default::default()
            },
        ],
    )
}

/// Puppet Strings — {3}. {2}, {T}: Tap or untap target creature.
pub fn puppet_strings() -> CardDefinition {
    artifact(
        "Puppet Strings",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::TapOrUntap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
    )
}

/// Mogg Cannon — {2}. {T}: fire a creature you control at the opponent; it
/// dies at the next end step.
pub fn mogg_cannon() -> CardDefinition {
    artifact(
        "Mogg Cannon",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Jinxed Idol — {2}. It bites its controller every upkeep; feed it a creature
/// to make it someone else's problem.
pub fn jinxed_idol() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::You),
                amount: Value::Const(2),
            },
        }],
        ..artifact(
            "Jinxed Idol",
            cost(&[generic(2)]),
            vec![ActivatedAbility {
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::Target(0)),
                    duration: Duration::Permanent,
                },
                ..Default::default()
            }],
        )
    }
}

// ── Artifact creatures ──────────────────────────────────────────────────────

/// Metallic Sliver — {1} 1/1, the cheapest way to turn on a Sliver lord.
pub fn metallic_sliver() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sliver],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        ..artifact("Metallic Sliver", cost(&[generic(1)]), vec![])
    }
}

/// Phyrexian Hulk — {6} 5/4.
pub fn phyrexian_hulk() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        ..artifact("Phyrexian Hulk", cost(&[generic(6)]), vec![])
    }
}

/// Patchwork Gnomes — {3} 2/1 that discards to regenerate.
pub fn patchwork_gnomes() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gnome], ..Default::default() },
        power: 2,
        toughness: 1,
        ..artifact(
            "Patchwork Gnomes",
            cost(&[generic(3)]),
            vec![ActivatedAbility {
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            }],
        )
    }
}

/// Squee's Toy — {1}. {T}: shave a point of damage off a creature.
pub fn squees_toy() -> CardDefinition {
    artifact(
        "Squee's Toy",
        cost(&[generic(1)]),
        vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
    )
}

/// "At the beginning of your upkeep, if `gate`, this artifact deals 1 damage to
/// target opponent or planeswalker." — Thumbscrews / Scalding Tongs.
fn upkeep_ping(name: &'static str, gate: Predicate) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource)
                .with_filter(gate),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)),
                amount: Value::ONE,
            },
        }],
        ..artifact(name, cost(&[generic(2)]), vec![])
    }
}

/// Thumbscrews — {2}. Pings while you're holding five or more cards.
pub fn thumbscrews() -> CardDefinition {
    upkeep_ping(
        "Thumbscrews",
        Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::Const(5)),
    )
}

/// Scalding Tongs — {2}. Pings while you're down to three cards or fewer.
pub fn scalding_tongs() -> CardDefinition {
    upkeep_ping(
        "Scalding Tongs",
        Predicate::Not(Box::new(Predicate::ValueAtLeast(
            Value::HandSizeOf(PlayerRef::You),
            Value::Const(4),
        ))),
    )
}

/// Torture Chamber — {3}. Pain counters accumulate and hurt you every end step,
/// until you cash them all in at a creature.
pub fn torture_chamber() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Pain,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::SelfSource,
                ),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::You),
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Pain,
                    },
                },
            },
        ],
        ..artifact(
            "Torture Chamber",
            cost(&[generic(3)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                remove_all_counters_cost: Some(CounterType::Pain),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::CountersRemovedAsCost,
                },
                ..Default::default()
            }],
        )
    }
}

/// Telethopter — {4} 3/1 that taps a friend for flight.
pub fn telethopter() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        power: 3,
        toughness: 1,
        ..artifact(
            "Telethopter",
            cost(&[generic(4)]),
            vec![ActivatedAbility {
                tap_others_cost: Some((R::Creature, 1)),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
        )
    }
}

/// Watchdog — {3} 1/2 that always blocks and dulls the whole attack while
/// untapped.
pub fn watchdog() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::MustBlock],
        static_abilities: vec![StaticAbility {
            description: "While untapped, creatures attacking you get -1/-0.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::Untapped },
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                    power: -1,
                    toughness: 0,
                }),
            },
        }],
        ..artifact("Watchdog", cost(&[generic(3)]), vec![])
    }
}
