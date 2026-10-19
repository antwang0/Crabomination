//! Commander: 2026-10-19 (`016Xh16H`) — Heidar, Rimewind Master's snow
//! tempo. Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, PlayerRef, Predicate, Selector, Value,
};
use crate::mana::{ManaCost, ManaSymbol, cost, generic, u, x};

fn snow(n: usize) -> ManaCost {
    ManaCost { symbols: vec![ManaSymbol::Snow; n] }
}

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

fn snow_creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        ..creature(name, mana, types, p, t)
    }
}

/// "Activate only if you control four or more snow permanents."
fn four_snow() -> Predicate {
    Predicate::ValueAtLeast(
        Value::count(Selector::EachPermanent(R::IsSnow.and(R::ControlledByYou))),
        Value::Const(4),
    )
}

/// Heidar, Rimewind Master — {4}{U} 3/3 legendary Human Wizard. {2}, {T}:
/// return target permanent to its owner's hand; only with four or more snow
/// permanents.
pub fn heidar_rimewind_master() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            condition: Some(four_snow()),
            effect: Effect::Move {
                what: target_filtered(R::Permanent),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..creature(
            "Heidar, Rimewind Master",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Rimewind Cryomancer — {3}{U} 2/3 Human Wizard. {1}, {T}: counter target
/// activated ability; only with four or more snow permanents.
pub fn rimewind_cryomancer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            condition: Some(four_snow()),
            effect: Effect::CounterAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::HasTriggeredAbilityOnStack.negate())),
            },
            ..Default::default()
        }],
        ..creature(
            "Rimewind Cryomancer",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Icehide Golem — {S} 2/2 snow artifact creature Golem.
pub fn icehide_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..snow_creature("Icehide Golem", snow(1), vec![CreatureType::Golem], 2, 2)
    }
}

/// Rimefeather Owl — {5}{U}{U} */* flying snow Bird, P/T the snow permanents
/// on the battlefield. {1}{S}: an ice counter on target permanent;
/// permanents with ice counters are snow.
pub fn rimefeather_owl() -> CardDefinition {
    // The ice counters' snow is this Owl's own grant, so it counts them
    // directly: the P/T is set mid-layer, before a layer-4 grant is readable.
    let snow_count = || Value::count(Selector::EachPermanent(R::IsSnow.or(R::WithCounter(CounterType::Ice))));
    let mut mana = cost(&[generic(1)]);
    mana.symbols.push(ManaSymbol::Snow);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Rimefeather Owl's power and toughness are each equal to the number of snow permanents on the battlefield.",
                effect: StaticEffect::SelfBasePtFromValue { power: snow_count(), toughness: snow_count() },
            },
            StaticAbility {
                description: "Permanents with ice counters on them are snow.",
                effect: StaticEffect::PermanentsWithCounterHaveSupertype {
                    counter: CounterType::Ice,
                    supertype: Supertype::Snow,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: mana,
            effect: Effect::AddCounter { what: target_filtered(R::Permanent), kind: CounterType::Ice, amount: Value::ONE },
            ..Default::default()
        }],
        ..snow_creature("Rimefeather Owl", cost(&[generic(5), u(), u()]), vec![CreatureType::Bird], 0, 0)
    }
}

/// Ascendant Spirit — {U} 1/1 snow Spirit. {S}{S}: a Spirit Warrior with
/// base 2/3. {S}{S}{S}: if a Warrior, a flying counter and a Spirit Warrior
/// Angel with base 4/4. {S}{S}{S}{S}: if an Angel, two +1/+1 counters and
/// "whenever this deals combat damage to a player, draw a card."
pub fn ascendant_spirit() -> CardDefinition {
    use CreatureType::{Angel, Spirit, Warrior};
    let become_ = |types: Vec<CreatureType>, p: i32, t: i32| {
        vec![
            Effect::BecomeCreatureType { what: Selector::This, creature_types: types, duration: Duration::Permanent },
            Effect::SetBasePT {
                what: Selector::This,
                power: Value::Const(p),
                toughness: Value::Const(t),
                duration: Duration::Permanent,
            },
        ]
    };
    let if_is = |t: CreatureType, then: Vec<Effect>| Effect::If {
        cond: Predicate::EntityMatches { what: Selector::This, filter: R::HasCreatureType(t) },
        then: Box::new(Effect::Seq(then)),
        else_: Box::new(Effect::Noop),
    };
    let mut angel = become_(vec![Spirit, Warrior, Angel], 4, 4);
    angel.insert(0, Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::Flying, amount: Value::ONE });
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: snow(2),
                effect: Effect::Seq(become_(vec![Spirit, Warrior], 2, 3)),
                ..Default::default()
            },
            ActivatedAbility { mana_cost: snow(3), effect: if_is(Warrior, angel), ..Default::default() },
            ActivatedAbility {
                mana_cost: snow(4),
                effect: if_is(
                    Angel,
                    vec![
                        Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                        Effect::GrantTriggeredAbility {
                            what: Selector::This,
                            trigger: Box::new(TriggeredAbility {
                                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                            }),
                            duration: Duration::Permanent,
                        },
                    ],
                ),
                ..Default::default()
            },
        ],
        ..snow_creature("Ascendant Spirit", cost(&[u()]), vec![Spirit], 1, 1)
    }
}

/// Pilfering Hawk — {1}{U} 1/2 flying snow Bird. {S}, {T}: draw, then
/// discard.
pub fn pilfering_hawk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: snow(1),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..snow_creature("Pilfering Hawk", cost(&[generic(1), u()]), vec![CreatureType::Bird], 1, 2)
    }
}

/// Berg Strider — {4}{U} 4/4 snow Giant Wizard. ETB tap target artifact or
/// creature an opponent controls; if {S} was spent to cast it, that
/// permanent skips its next untap.
pub fn berg_strider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByOpponent)) },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::SnowManaSpentToCastSource, Value::ONE),
                then: Box::new(Effect::SkipNextUntap { what: Selector::Target(0) }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..snow_creature(
            "Berg Strider",
            cost(&[generic(4), u()]),
            vec![CreatureType::Giant, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Avalanche Caller — {1}{U} 1/3 snow Human Wizard. {2}: target snow land
/// you control becomes a 4/4 Elemental creature with hexproof and haste
/// until end of turn.
pub fn avalanche_caller() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeCreature {
                what: target_filtered(R::IsSnow.and(R::Land).and(R::ControlledByYou)),
                power: Value::Const(4),
                toughness: Value::Const(4),
                creature_types: vec![CreatureType::Elemental],
                keywords: vec![Keyword::Hexproof, Keyword::Haste],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..snow_creature(
            "Avalanche Caller",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Blizzard Strix — {4}{U} 3/2 flash, flying snow Bird. ETB, if you control
/// another snow permanent, exile target permanent other than it; it returns
/// under its owner's control at the next end step.
pub fn blizzard_strix() -> CardDefinition {
    let mut t = etb(Effect::ExileReturnNextEndStep { what: target_filtered(R::Permanent.and(R::OtherThanSource)) });
    t.event = t.event.with_filter(Predicate::ValueAtLeast(
        Value::count(Selector::EachPermanent(R::IsSnow.and(R::ControlledByYou).and(R::OtherThanSource))),
        Value::ONE,
    ));
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![t],
        ..snow_creature("Blizzard Strix", cost(&[generic(4), u()]), vec![CreatureType::Bird], 3, 2)
    }
}

/// Winter's Chill — {X}{U} instant, during combat before blockers. For each
/// of X target attacking creatures, unless its controller pays {2}, destroy
/// it at end of combat.
pub fn winters_chill() -> CardDefinition {
    CardDefinition {
        name: "Winter's Chill",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Instant],
        cast_condition: Some(Predicate::Any(vec![
            Predicate::CurrentStepIs(crate::game::types::TurnStep::BeginCombat),
            Predicate::CurrentStepIs(crate::game::types::TurnStep::DeclareAttackers),
        ])),
        effect: Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 1,
                filter: R::Creature.and(R::IsAttacking),
                effect: Box::new(Effect::UnlessPlayerPays {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    cost: WardCost::Mana(cost(&[generic(2)])),
                    then: Box::new(Effect::DelayUntilWithCapture {
                        kind: DelayedTriggerKind::EndOfCombat,
                        capture: Selector::Target(0),
                        body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                    }),
                    if_paid: None,
                }),
            }),
        },
        ..Default::default()
    }
}
