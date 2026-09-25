//! Commander: the cards the **Counter Intelligence** precon (EOC, Inspirit,
//! Flagship Vessel) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_inspirit.rs`.
//!
//! Residuals (each also on its card):
//! - **Cloud Key** — the card-type choice also offers land and planeswalker.
//! - **Inspirit, Flagship Vessel** / **Depthshaker Titan** — "up to one" /
//!   "any number of" targets take what the targeter picks; Inspirit always
//!   targets when it can.
//! - **Moxite Refinery** — the X counters may come from among several of your
//!   artifacts and creatures, not from one.
//! - **Resourceful Defense** — "any number of counters" moves all of them.
//! - **Ripples of Potential** — the phase-out is all or none of your
//!   permanents with counters, not a pick among those proliferated.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, StationBand, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, station, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, r, u, w, x, ManaCost};

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn charge(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::Charge, amount: Value::Const(n) }
}

fn charges_on_self() -> Value {
    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge }
}

/// "Whenever you cast [filter] spell".
fn on_cast(filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(filter)),
        effect,
    }
}

/// "At the beginning of combat on your turn".
fn your_combat(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl), effect }
}

fn artifact_card() -> R {
    R::HasCardType(CardType::Artifact)
}

/// Inspirit, Flagship Vessel — Spacecraft, station. 1+: at the beginning of
/// combat on your turn, a +1/+1 counter or two charge counters on up to one
/// other target artifact. 8+: a 5/5 flier; your other artifacts have hexproof
/// and indestructible. ⚠ Always targets when it can.
pub fn inspirit_flagship_vessel() -> CardDefinition {
    let other_artifacts = || yours(artifact_card().and(R::OtherThanSource));
    let grant = |keyword| StaticEffect::GrantKeyword { applies_to: other_artifacts(), keyword };
    let slot = || target_filtered(artifact_card().and(R::OtherThanSource));
    CardDefinition {
        name: "Inspirit, Flagship Vessel",
        cost: cost(&[u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: vec![
            StationBand {
                min: 1,
                triggers: vec![your_combat(Effect::ChooseMode(vec![
                    Effect::AddCounter { what: slot(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    charge(slot(), 2),
                ]))],
                ..Default::default()
            },
            StationBand {
                min: 8,
                keywords: vec![Keyword::Flying],
                pt: Some((5, 5)),
                statics: vec![grant(Keyword::Hexproof), grant(Keyword::Indestructible)],
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kilo, Apogee Mind — haste; whenever it becomes tapped, proliferate.
pub fn kilo_apogee_mind() -> CardDefinition {
    CardDefinition {
        name: "Kilo, Apogee Mind",
        cost: cost(&[u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot, CreatureType::Artificer], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Chrome Host Seedshark — flying; a noncreature spell incubates X, its
/// mana value.
pub fn chrome_host_seedshark() -> CardDefinition {
    CardDefinition {
        name: "Chrome Host Seedshark",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian, CreatureType::Shark], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_cast(
            R::Not(Box::new(R::Creature)),
            Effect::Incubate {
                who: PlayerRef::You,
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        )],
        ..Default::default()
    }
}

/// Cloud Key — as it enters, choose a card type; your spells of it cost {1}
/// less. ⚠ The choice also offers land and planeswalker.
pub fn cloud_key() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseCardTypeForSource),
        static_abilities: vec![StaticAbility {
            description: "Spells you cast of the chosen type cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::IsSourceChosenCardType, amount: 1 },
        }],
        ..artifact("Cloud Key", cost(&[generic(3)]))
    }
}

/// Depthshaker Titan — any number of your noncreature artifacts become 3/3
/// artifact creatures, sacrificed at the next end step; your artifact
/// creatures have melee, trample and haste.
pub fn depthshaker_titan() -> CardDefinition {
    let grant = |keyword| StaticAbility {
        description: "Each artifact creature you control has melee, trample, and haste.",
        effect: StaticEffect::GrantKeyword { applies_to: yours(R::Creature.and(artifact_card())), keyword },
    };
    CardDefinition {
        name: "Depthshaker Titan",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 20,
            min_targets: 0,
            filter: artifact_card().and(R::Not(Box::new(R::Creature))).and(R::ControlledByYou),
            effect: Box::new(Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: Selector::Target(0),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::Permanent,
                },
                Effect::SacrificeAtNextEndStep { what: Selector::Target(0) },
            ])),
        })],
        static_abilities: vec![grant(Keyword::Melee), grant(Keyword::Trample), grant(Keyword::Haste)],
        ..Default::default()
    }
}

/// Empowered Autogenerator — enters tapped; {T}: a charge counter, then X mana
/// of one color, X = its charge counters.
pub fn empowered_autogenerator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                charge(Selector::This, 1),
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(charges_on_self()) },
            ]),
            ..Default::default()
        }],
        ..artifact("Empowered Autogenerator", cost(&[generic(4)]))
    }
}

/// Gavel of the Righteous — a charge counter each combat on your turn; +1/+1
/// per counter of any kind on it, double strike at four or more; equip {3} or
/// remove a counter from it.
pub fn gavel_of_the_righteous() -> CardDefinition {
    let equipped = || Selector::AttachedTo(Box::new(Selector::This));
    let counters = || Value::TotalCountersOn { what: Box::new(Selector::This) };
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![your_combat(charge(Selector::This, 1))],
        static_abilities: vec![
            StaticAbility {
                description: "Equipped creature gets +1/+1 for each counter on this Equipment.",
                effect: StaticEffect::PumpPTByValue { applies_to: equipped(), power: counters(), toughness: counters() },
            },
            StaticAbility {
                description: "As long as this Equipment has four or more counters on it, equipped creature has double strike.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::ValueAtLeast(counters(), Value::Const(4)),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: equipped(), keyword: Keyword::DoubleStrike }),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            remove_counter_among_filter: Some((None, 1, R::IsSource)),
            effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
            ..Default::default()
        }],
        ..artifact("Gavel of the Righteous", cost(&[generic(2)]))
    }
}

/// Golem Foundry — an artifact spell may add a charge counter; remove three:
/// a 3/3 Golem.
pub fn golem_foundry() -> CardDefinition {
    let golem = crabomination_base::tokens::golem_3_3_token();
    CardDefinition {
        triggered_abilities: vec![on_cast(
            artifact_card(),
            Effect::MayDo { description: "Put a charge counter on Golem Foundry?".into(), body: Box::new(charge(Selector::This, 1)) },
        )],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Charge, 3)),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(golem) },
            ..Default::default()
        }],
        ..artifact("Golem Foundry", cost(&[generic(3)]))
    }
}

/// Insight Engine — {2}, {T}: a charge counter, then draw per charge counter.
pub fn insight_engine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                charge(Selector::This, 1),
                Effect::Draw { who: Selector::You, amount: charges_on_self() },
            ]),
            ..Default::default()
        }],
        ..artifact("Insight Engine", cost(&[generic(2), u()]))
    }
}

/// Long-Range Sensor — a charge counter per player you attack; {1}, remove two:
/// discover 4 (sorcery speed).
pub fn long_range_sensor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::OpponentsAttackedThisCombat,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            remove_counter_cost: Some((CounterType::Charge, 2)),
            effect: Effect::Discover { n: Value::Const(4), filter: None },
            ..Default::default()
        }],
        ..artifact("Long-Range Sensor", cost(&[generic(2), r()]))
    }
}

/// Lux Artillery — your artifact creature spells gain sunburst; at your end
/// step, thirty or more counters among your artifacts and creatures deal 10
/// to each opponent.
pub fn lux_artillery() -> CardDefinition {
    let mine = || yours(artifact_card().or(R::Creature));
    CardDefinition {
        triggered_abilities: vec![
            on_cast(R::Creature.and(artifact_card()), Effect::SpellGainsSunburst { what: Selector::TriggerSource }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                    Predicate::ValueAtLeast(Value::TotalCountersOn { what: Box::new(mine()) }, Value::Const(30)),
                ),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(10) },
            },
        ],
        ..artifact("Lux Artillery", cost(&[generic(4)]))
    }
}

/// Lux Cannon — {T}: a charge counter; {T}, remove three: destroy target
/// permanent.
pub fn lux_cannon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: charge(Selector::This, 1), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 3)),
                effect: Effect::Destroy { what: target_filtered(R::Permanent) },
                ..Default::default()
            },
        ],
        ..artifact("Lux Cannon", cost(&[generic(4)]))
    }
}

/// Moxite Refinery — {2}, {T}, remove X counters from your artifacts and
/// creatures: X charge counters on target artifact, or X +1/+1 counters on
/// target creature (sorcery speed). ⚠ The X may come from several permanents.
pub fn moxite_refinery() -> CardDefinition {
    let mode = |filter: R, kind| ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        tap_cost: true,
        sorcery_speed: true,
        remove_counter_among_x: Some((None, artifact_card().or(R::Creature))),
        effect: Effect::AddCounter { what: target_filtered(filter), kind, amount: Value::XFromCost },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            mode(artifact_card(), CounterType::Charge),
            mode(R::Creature, CounterType::PlusOnePlusOne),
        ],
        ..artifact("Moxite Refinery", cost(&[generic(2)]))
    }
}

/// Patrolling Peacemaker — enters with two +1/+1 counters; an opponent
/// committing a crime proliferates.
pub fn patrolling_peacemaker() -> CardDefinition {
    CardDefinition {
        name: "Patrolling Peacemaker",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot, CreatureType::Soldier], ..Default::default() },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::OpponentControl),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Resourceful Defense — a permanent of yours leaving with counters puts them
/// on target permanent you control; {4}{W}: move counters from one of your
/// permanents onto another. ⚠ The move takes every counter.
pub fn resourceful_defense() -> CardDefinition {
    let yours_target = |slot| Selector::TargetFiltered { slot, filter: R::Permanent.and(R::ControlledByYou) };
    CardDefinition {
        name: "Resourceful Defense",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(Value::TotalCountersOn { what: Box::new(Selector::TriggerSource) }, Value::ONE),
            ),
            effect: Effect::PutCountersOf { from: Selector::TriggerSource, to: yours_target(0) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::MoveAllCounters { from: yours_target(0), to: yours_target(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ripples of Potential — proliferate, then you may phase out your permanents
/// with counters. ⚠ All or none of them.
pub fn ripples_of_potential() -> CardDefinition {
    CardDefinition {
        name: "Ripples of Potential",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Proliferate,
            Effect::MayDo {
                description: "Phase out your permanents that got a counter?".into(),
                body: Box::new(Effect::PhaseOut {
                    what: yours(R::Permanent.and(R::WithAnyCounter)),
                    until_source_leaves: false,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Solar Array — {T}: one mana of any color; your next artifact spell this
/// turn gains sunburst.
pub fn solar_array() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                Effect::OnYourNextSpellOfTypeThisTurn {
                    card_type: CardType::Artifact,
                    body: Box::new(Effect::SpellGainsSunburst { what: Selector::TriggerSource }),
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Solar Array", cost(&[generic(3)]))
    }
}

/// Surge Conductor — another nontoken artifact of yours entering proliferates.
pub fn surge_conductor() -> CardDefinition {
    CardDefinition {
        name: "Surge Conductor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: artifact_card().and(R::Not(Box::new(R::IsToken))),
                },
            ),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Universal Surveillance — improvise; draw X.
pub fn universal_surveillance() -> CardDefinition {
    CardDefinition {
        name: "Universal Surveillance",
        cost: cost(&[x(), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Improvise],
        effect: Effect::Draw { who: Selector::You, amount: Value::XFromCost },
        ..Default::default()
    }
}

/// Uthros Research Craft — Spacecraft, station. 3+: an artifact spell draws a
/// card and charges it. 12+: a 0/8 flier with +1/+0 per artifact you control.
pub fn uthros_research_craft() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: vec![
            StationBand {
                min: 3,
                triggers: vec![on_cast(
                    artifact_card(),
                    Effect::Seq(vec![Effect::Draw { who: Selector::You, amount: Value::ONE }, charge(Selector::This, 1)]),
                )],
                ..Default::default()
            },
            StationBand {
                min: 12,
                keywords: vec![Keyword::Flying],
                pt: Some((0, 8)),
                statics: vec![StaticEffect::PumpPTByValue {
                    applies_to: Selector::This,
                    power: Value::CountOf(Box::new(yours(artifact_card()))),
                    toughness: Value::Const(0),
                }],
                ..Default::default()
            },
        ],
        ..artifact("Uthros Research Craft", cost(&[generic(2), u()]))
    }
}
