//! Legacy prison/combo staples (TODO.md deferred list): Squee, Dark Depths,
//! Smokestack, Tangle Wire. Tests in `tests/recent105.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::mana::{Color, cost, generic, r};

/// Squee, the Immortal — {1}{R}{R} 2/1 Goblin. Castable from graveyard or
/// exile (modeled as pay-cost Move activations, like Gravecrawler).
pub fn squee_the_immortal() -> CardDefinition {
    let recast = |from_graveyard: bool| ActivatedAbility {
        mana_cost: cost(&[generic(1), r(), r()]),
        effect: Effect::Move {
            what: Selector::This,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        sorcery_speed: true,
        from_graveyard,
        from_exile: !from_graveyard,
        ..Default::default()
    };
    CardDefinition {
        name: "Squee, the Immortal",
        cost: cost(&[generic(1), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![recast(true), recast(false)],
        ..Default::default()
    }
}

/// Dark Depths — Legendary Snow Land; ten ice counters; {3}: remove one.
/// No counters left: sacrifice it for Marit Lage (20/20 flying indestructible).
pub fn dark_depths() -> CardDefinition {
    CardDefinition {
        name: "Dark Depths",
        supertypes: vec![Supertype::Legendary, Supertype::Snow],
        card_types: vec![CardType::Land],
        enters_with_counters: Some((CounterType::Ice, Value::Const(10))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Ice,
                    amount: Value::ONE,
                },
                // The "when it has no ice counters" state trigger is folded
                // into the removal.
                Effect::If {
                    cond: crate::effect::Predicate::ValueAtMost(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Ice,
                        },
                        Value::Const(0),
                    ),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Box::new(marit_lage()),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn marit_lage() -> TokenDefinition {
    TokenDefinition {
        name: "Marit Lage".into(),
        power: 20,
        toughness: 20,
        keywords: vec![Keyword::Flying, Keyword::Indestructible],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Smokestack — {4} Artifact. Your upkeep: may add a soot counter; each
/// player's upkeep: that player sacrifices a permanent per soot counter.
pub fn smokestack() -> CardDefinition {
    CardDefinition {
        name: "Smokestack",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayDo {
                    description: "Put a soot counter on Smokestack?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Soot,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    count: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Soot,
                    },
                    filter: SelectionRequirement::Permanent,
                },
            },
        ],
        ..Default::default()
    }
}

/// Tangle Wire — {3} Artifact, Fading 4. Each player's upkeep: that player
/// taps an untapped artifact/creature/land per fade counter on this.
pub fn tangle_wire() -> CardDefinition {
    CardDefinition {
        name: "Tangle Wire",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Fading(4)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::PlayerTapsUntapped {
                who: PlayerRef::ActivePlayer,
                filter: SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .or(SelectionRequirement::Land),
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Fade,
                },
            },
        }],
        ..Default::default()
    }
}
