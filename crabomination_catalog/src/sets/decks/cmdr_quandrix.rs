//! Commander: the cards the **Quantum Quandrix** precon (C21, Adrix and Nev,
//! Twincasters) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_quandrix.rs`.
//!
//! Residuals (each also on its card):
//! - **Esix, Fractal Bloom** — the copied creature is the engine's pick (the
//!   greatest mana value, anyone's), and the "may" is always taken.
//! - **Primal Empathy** — the +1/+1 counter goes on your creature of greatest
//!   power (the engine's pick).
//! - **Ruxa, Patient Professor** — "no abilities" reads printed abilities, and
//!   the unblocked-damage option is always taken.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::catalog::sets::sos::fractal_token;
use crate::effect::shortcut::{etb, magecraft, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, cost, g, generic, u, x};
use std::sync::Arc;

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn green_token(name: &str, types: Vec<CreatureType>, p: i32, t: i32) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

/// A 0/0 Fractal with `n` +1/+1 counters (CR 614.1c — they enter with it).
fn fractal(n: Value) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(fractal_token().entering_with(CounterType::PlusOnePlusOne, n)),
    }
}

/// Commander's Insight — target player draws X, plus one for each time they
/// have cast a commander from the command zone this game.
pub fn commanders_insight() -> CardDefinition {
    spell(
        "Commander's Insight",
        cost(&[x(), u(), u(), u()]),
        CardType::Instant,
        Effect::Draw {
            who: target_filtered(R::Player),
            amount: Value::Sum(vec![
                Value::XFromCost,
                Value::CommanderCastsFromCommandZone(PlayerRef::Target(0)),
            ]),
        },
    )
}

/// Crafty Cutpurse — flash; on entering, tokens opponents would create this
/// turn are created under your control instead (CR 614).
pub fn crafty_cutpurse() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::StealOpponentTokensThisTurn)],
        ..creature("Crafty Cutpurse", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 2)
    }
}

/// Curiosity Crafter — flying; no maximum hand size; a creature token of
/// yours dealing combat damage to a player draws a card.
pub fn curiosity_crafter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::IsToken) },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Curiosity Crafter", cost(&[generic(3), u()]), vec![CreatureType::Bird, CreatureType::Wizard], 3, 3)
    }
}

/// Deekah, Fractal Theorist — magecraft: a Fractal with counters equal to the
/// spell's mana value; {3}{U}: target creature token can't be blocked.
pub fn deekah_fractal_theorist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![magecraft(fractal(Value::ManaValueOf(Box::new(Selector::TriggerSource))))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::IsToken)),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Deekah, Fractal Theorist",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Esix, Fractal Bloom — flying; the first token batch on each of your turns
/// becomes copies of another creature (the engine picks the greatest mana
/// value; the "may" is always taken).
pub fn esix_fractal_bloom() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "The first time you would create tokens during each of your turns, \
                          create copies of another creature instead.",
            effect: StaticEffect::FirstTokensOnYourTurnBecomeCopiesOfChosen,
        }],
        ..creature("Esix, Fractal Bloom", cost(&[generic(4), g(), u()]), vec![CreatureType::Fractal], 4, 4)
    }
}

/// Fractal Harness — a Fractal with X counters, equipped on entry; attacking
/// doubles its +1/+1 counters. Equip {2}.
pub fn fractal_harness() -> CardDefinition {
    CardDefinition {
        name: "Fractal Harness",
        cost: cost(&[x(), generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            fractal(Value::XFromCost),
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::DoubleCountersOnEach { what: Selector::This, kind: CounterType::PlusOnePlusOne },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Garruk, Primal Hunter — +1: a 3/3 Beast; −3: draw the greatest power among
/// your creatures; −6: a 6/6 Wurm for each land you control.
pub fn garruk_primal_hunter() -> CardDefinition {
    CardDefinition {
        name: "Garruk, Primal Hunter",
        cost: cost(&[generic(2), g(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Garruk], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: green_token("Beast", vec![CreatureType::Beast], 3, 3),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::GreatestPowerControlled { who: PlayerRef::You },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountOf(Box::new(Selector::EachPermanent(R::Land.and(R::ControlledByYou)))),
                    definition: green_token("Wurm", vec![CreatureType::Wurm], 6, 6),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Guardian Augmenter — flash; commander creatures you control get +2/+2 and
/// commanders you control have hexproof.
pub fn guardian_augmenter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![
            StaticAbility {
                description: "Commander creatures you control get +2/+2.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature.and(R::IsCommander),
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                    opponents: false,
                    all_players: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
            StaticAbility {
                description: "Commanders you control have hexproof.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::IsCommander,
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Hexproof],
                    opponents: false,
                    all_players: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
        ],
        ..creature("Guardian Augmenter", cost(&[generic(2), g()]), vec![CreatureType::Troll, CreatureType::Wizard], 2, 2)
    }
}

/// Kazandu Tuskcaller — level up {1}{G} (CR 702.87); level 2-5: {T}: a 3/3
/// Elephant; level 6+: {T}: two.
pub fn kazandu_tuskcaller() -> CardDefinition {
    let level_at_least = |n: u32| Predicate::SourceHasCountersAtLeast { counter: CounterType::Level, n };
    let elephants = |n: i32| Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: green_token("Elephant", vec![CreatureType::Elephant], 3, 3),
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                sorcery_speed: true,
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Level, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::All(vec![level_at_least(2), Predicate::Not(Box::new(level_at_least(6)))])),
                effect: elephants(1),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(level_at_least(6)),
                effect: elephants(2),
                ..Default::default()
            },
        ],
        ..creature("Kazandu Tuskcaller", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Shaman], 1, 1)
    }
}

/// Oversimplify — exile all creatures; each player gets a Fractal with the
/// total power of the creatures they lost.
pub fn oversimplify() -> CardDefinition {
    spell(
        "Oversimplify",
        cost(&[generic(3), g(), u()]),
        CardType::Sorcery,
        Effect::ExileAllThenTokenPerPlayerByPower { filter: R::Creature, definition: Arc::new(fractal_token()) },
    )
}

/// Paradox Zone — enters with a growth counter; at your end step, double them
/// and make a Fractal with that many +1/+1 counters.
pub fn paradox_zone() -> CardDefinition {
    CardDefinition {
        name: "Paradox Zone",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Growth, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::DoubleCountersOnEach { what: Selector::This, kind: CounterType::Growth },
                fractal(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Growth }),
            ]),
        }],
        ..Default::default()
    }
}

/// Primal Empathy — at your upkeep, draw if you control a creature of the
/// greatest power on the battlefield; otherwise a +1/+1 counter on your
/// creature of greatest power (the engine's pick).
pub fn primal_empathy() -> CardDefinition {
    CardDefinition {
        name: "Primal Empathy",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ControlsGreatestPowerCreature { who: PlayerRef::You },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::AddCounter {
                    what: Selector::GreatestPowerYouControl,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Ruxa, Patient Professor — entering or attacking returns a creature card
/// with no abilities from your graveyard to hand; your creatures with no
/// abilities get +1/+1 and assign combat damage as though unblocked.
pub fn ruxa_patient_professor() -> CardDefinition {
    let regrow = || Effect::Move {
        what: target_filtered(R::Creature.and(R::HasNoAbilities).from_your_graveyard()),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(regrow()),
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: regrow() },
        ],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control with no abilities get +1/+1 and may assign combat \
                              damage as though they weren't blocked.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature.and(R::HasNoAbilities),
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::AssignsDamageAsThoughUnblocked],
                    opponents: false,
                    all_players: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
        ],
        ..creature(
            "Ruxa, Patient Professor",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Bear, CreatureType::Druid],
            4,
            4,
        )
    }
}

/// Spawning Kraken — a Kraken, Leviathan, Octopus or Serpent of yours dealing
/// combat damage to a player makes a 9/9 Kraken.
pub fn spawning_kraken() -> CardDefinition {
    let sea = R::HasCreatureType(CreatureType::Kraken)
        .or(R::HasCreatureType(CreatureType::Leviathan))
        .or(R::HasCreatureType(CreatureType::Octopus))
        .or(R::HasCreatureType(CreatureType::Serpent));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: sea }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Kraken".into(),
                    power: 9,
                    toughness: 9,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Kraken], ..Default::default() },
                    ..Default::default()
                }),
            },
        }],
        ..creature("Spawning Kraken", cost(&[generic(5), u()]), vec![CreatureType::Kraken], 6, 6)
    }
}

/// Study Hall — {T}: {C}; {1}, {T}: any color, and spent on your commander it
/// scries X, X its command-zone casts (`SpendRestriction::CommanderCastScry`).
pub fn study_hall() -> CardDefinition {
    CardDefinition {
        name: "Study Hall",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Restricted(
                        Box::new(crate::effect::ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::CommanderCastScry,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Theoretical Duplication — this turn, each nontoken creature an opponent
/// controls that enters is copied for you.
pub fn theoretical_duplication() -> CardDefinition {
    spell(
        "Theoretical Duplication",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::WheneverCreatureEntersThisTurn {
            filter: R::Creature.and(R::ControlledByOpponent).and(R::NotToken),
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
        },
    )
}
