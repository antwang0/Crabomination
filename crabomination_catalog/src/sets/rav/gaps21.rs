//! Ravnica (RAV) gap wave 21 — the graveyard/exile-flavoured rares plus a few
//! combat tricks. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, Value, WardCost,
};
use crate::effect::shortcut::{target_filtered, transmute};
use crate::effect::{
    DelayedTriggerKind, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector,
    StaticEffect, TriggeredAbility, ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, hybrid, u, w};

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Blood Funnel — {1}{B} Enchantment. Noncreature spells you cast cost {2}
/// less; each one is countered unless you sacrifice a creature.
pub fn blood_funnel() -> CardDefinition {
    CardDefinition {
        name: "Blood Funnel",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells you cast cost {2} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::Creature.negate(),
                amount: 2,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.negate(),
                }),
            effect: Effect::CounterUnless {
                what: Selector::TriggerSource,
                cost: WardCost::SacrificeCreature,
            },
        }],
        ..Default::default()
    }
}

/// Bottled Cloister — {4} Artifact. Your hand is exiled under it during each
/// opponent's upkeep and comes back on yours, plus a card.
pub fn bottled_cloister() -> CardDefinition {
    CardDefinition {
        name: "Bottled Cloister",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::OpponentControl,
                ),
                effect: Effect::ExileHandLinked,
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Seq(vec![
                    Effect::ReturnLinkedExilesToHand,
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Crown of Convergence — {2} Artifact. Play with the top card of your library
/// revealed; while it's a creature card, your creatures sharing a color with
/// it get +1/+1. {G}{W}: bottom the top card.
pub fn crown_of_convergence() -> CardDefinition {
    CardDefinition {
        name: "Crown of Convergence",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Play with the top card of your library revealed.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "Creatures sharing a color with a revealed creature card get +1/+1.",
                effect: StaticEffect::AnthemForColorSharedWithLibraryTop {
                    power: 1,
                    toughness: 1,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), w()]),
            effect: Effect::PutTopOnBottom { who: Selector::You },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dimir Doppelganger — {1}{U}{B} 0/2. {1}{U}{B}: exile a creature card from a
/// graveyard and become a copy of it, keeping this ability.
pub fn dimir_doppelganger() -> CardDefinition {
    CardDefinition {
        name: "Dimir Doppelganger",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Shapeshifter]),
        power: 0,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u(), b()]),
            effect: Effect::ExileFromGraveyardBecomeCopy {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dimir Machinations — {2}{B} Sorcery. Look at the top three cards of target
/// player's library and exile any number. Transmute {1}{B}{B}.
pub fn dimir_machinations() -> CardDefinition {
    CardDefinition {
        name: "Dimir Machinations",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookExileAnyNumberRestBack {
            who: target_filtered(R::Player),
            count: Value::Const(3),
        },
        activated_abilities: vec![transmute(cost(&[generic(1), b(), b()]), 3)],
        ..Default::default()
    }
}

/// Bloodbond March — {2}{B}{G} Enchantment. Whenever a player casts a creature
/// spell, every graveyard gives back all cards with that name.
pub fn bloodbond_march() -> CardDefinition {
    CardDefinition {
        name: "Bloodbond March",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::ReturnSameNameFromAllGraveyards {
                what: Selector::TriggerSource,
            },
        }],
        ..Default::default()
    }
}

/// Gaze of the Gorgon — {3}{B/G} Instant. Regenerate target creature; at this
/// turn's next end of combat, destroy everything it fought.
pub fn gaze_of_the_gorgon() -> CardDefinition {
    CardDefinition {
        name: "Gaze of the Gorgon",
        cost: cost(&[generic(3), hybrid(Color::Black, Color::Green)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Regenerate {
                what: target_filtered(R::Creature),
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::EndOfCombat,
                body: Box::new(Effect::Destroy {
                    what: Selector::CreaturesInCombatWith(Box::new(Selector::Target(0))),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Sisters of Stone Death — {4}{B}{B}{G}{G} 7/5. Lures a blocker, eats what it
/// fights, and redeploys the exiled bodies.
pub fn sisters_of_stone_death() -> CardDefinition {
    CardDefinition {
        name: "Sisters of Stone Death",
        cost: cost(&[generic(4), b(), b(), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: types(vec![CreatureType::Gorgon]),
        power: 7,
        toughness: 5,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::MustBlockSource {
                    what: target_filtered(R::Creature),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b(), g()]),
                effect: Effect::ExileTaggedWithSource {
                    what: target_filtered(R::Creature.and(R::InCombatWithSource)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                effect: Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
